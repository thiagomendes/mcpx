#![allow(dead_code)]
//! Health Check Service
//! 
//! Periodic job that checks connectivity to all registered MCP servers.
//! Uses rmcp SDK for proper MCP protocol communication.

use crate::services::db::Database;
use crate::services::crypto;
use chrono::Utc;
use rmcp::{
    ServiceExt,
    model::{ClientCapabilities, ClientInfo, Implementation},
    transport::streamable_http_client::{StreamableHttpClientTransport, StreamableHttpClientTransportConfig},
    service::RunningService,
};
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

const SQL_SELECT_CREDENTIAL: &str = "SELECT encrypted_value FROM credentials WHERE server_id = $1 AND credential_type = $2";

#[derive(Debug, sqlx::FromRow)]
struct ServerForHealthCheck {
    id: Uuid,
    org_id: Uuid,
    url: String,
    auth_type: Option<String>,
    status: Option<String>,
    oauth_client_id: Option<String>,
    oauth_token_url: Option<String>,
}

pub async fn start_health_check_job(db: Database, encryption_key: String, interval_seconds: u64) {
    tracing::info!("Starting health check job with {}s interval", interval_seconds);
    
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_seconds));
        
        loop {
            ticker.tick().await;
            tracing::info!("Running health check cycle...");
            
            if let Err(e) = run_health_check_cycle(&db, &encryption_key).await {
                tracing::error!("Health check cycle failed: {:?}", e);
            }
        }
    });
}

async fn run_health_check_cycle(db: &Database, encryption_key: &str) -> Result<(), String> {

    let servers: Vec<ServerForHealthCheck> = sqlx::query_as(
        "SELECT id, org_id, url, auth_type, status, oauth_client_id, oauth_token_url FROM servers WHERE status != 'pending_auth' AND enabled = true"
    )
    .fetch_all(&db.pool)
    .await
    .map_err(|e| format!("Failed to fetch servers: {}", e))?;
    
    tracing::info!("Checking {} servers...", servers.len());
    
    for server in servers {
        let result = check_single_server(db, &server, encryption_key).await;
        
        match result {
            Ok(()) => {
                tracing::info!("Server {} is healthy", server.id);
            }
            Err(e) => {
                tracing::warn!("Server {} is unhealthy: {}", server.id, e);
            }
        }
    }
    
    Ok(())
}

async fn check_single_server(db: &Database, server: &ServerForHealthCheck, encryption_key: &str) -> Result<(), String> {
    // Fetch credentials based on auth_type
    let (auth_header_name, auth_header_value): (Option<String>, Option<String>) = 
        match server.auth_type.as_deref() {
            Some("api_key") => {
                let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_CREDENTIAL)
                    .bind(server.id)
                    .bind("api_key")
                    .fetch_optional(&db.pool)
                    .await
                    .map_err(|e| format!("DB error: {}", e))?;
                
                if let Some((encrypted,)) = row {
                    let key = crypto::derive_key(encryption_key);
                    match crypto::decrypt(&encrypted, &key) {
                        Ok(api_key) => (Some("X-API-Key".to_string()), Some(api_key)),
                        Err(_) => (None, None)
                    }
                } else {
                    (None, None)
                }
            }
            Some("bearer") => {
                let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_CREDENTIAL)
                    .bind(server.id)
                    .bind("bearer")
                    .fetch_optional(&db.pool)
                    .await
                    .map_err(|e| format!("DB error: {}", e))?;
                
                if let Some((encrypted,)) = row {
                    let key = crypto::derive_key(encryption_key);
                    match crypto::decrypt(&encrypted, &key) {
                        Ok(token) => (Some("Authorization".to_string()), Some(format!("Bearer {}", token))),
                        Err(_) => (None, None)
                    }
                } else {
                    (None, None)
                }
            }
            Some("oauth_client_credentials") => {
                // Get client_id and token_url from server record
                let client_id = server.oauth_client_id.clone().unwrap_or_default();
                let token_url = server.oauth_token_url.clone().unwrap_or_default();
                
                // Get client_secret from credentials
                let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_CREDENTIAL)
                    .bind(server.id)
                    .bind("oauth_client_secret")
                    .fetch_optional(&db.pool)
                    .await
                    .map_err(|e| format!("DB error: {}", e))?;
                
                let client_secret = if let Some((encrypted,)) = row {
                    let key = crypto::derive_key(encryption_key);
                    crypto::decrypt(&encrypted, &key).ok()
                } else {
                    None
                };
                
                if client_id.is_empty() || token_url.is_empty() {
                    (None, None)
                } else if let Some(secret) = client_secret {
                    // Fetch token from token endpoint
                    let client = reqwest::Client::new();
                    let token_response = client.post(&token_url)
                        .json(&serde_json::json!({
                            "grant_type": "client_credentials",
                            "client_id": client_id,
                            "client_secret": secret
                        }))
                        .send()
                        .await;
                    
                    match token_response {
                        Ok(resp) if resp.status().is_success() => {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
                                    (Some("Authorization".to_string()), Some(format!("Bearer {}", access_token)))
                                } else {
                                    (None, None)
                                }
                            } else {
                                (None, None)
                            }
                        }
                        _ => (None, None)
                    }
                } else {
                   (None, None)
                }
            }
            Some("oauth_auto") => {
                // For health check, we use server-level token lookup (no user context)
                match get_access_token(db, server.id, encryption_key).await {
                    Ok(Some(token)) => (Some("Authorization".to_string()), Some(format!("Bearer {}", token))),
                    _ => (None, None)
                }
            }
            _ => (None, None)
        };

    let result = test_mcp_connection(&server.url, auth_header_name.as_deref(), auth_header_value.as_deref()).await;
    
    match result {
        Ok(_) => {
            sqlx::query(
                "UPDATE servers SET status = 'healthy', last_health_check = NOW(), health_error = NULL WHERE id = $1"
            )
            .bind(server.id)
            .execute(&db.pool)
            .await
            .map_err(|e| format!("Failed to update server status: {}", e))?;
            
            Ok(())
        }
        Err(e) => {
            let (new_status, error_msg) = if e.contains("invalid_token") || e.contains("Token is not active") {
                match try_refresh_token(db, server.id, encryption_key).await {
                    Ok(new_token) => {
                        match test_mcp_connection(&server.url, Some("Authorization"), Some(&format!("Bearer {}", new_token))).await {
                            Ok(_) => {
                                ("healthy".to_string(), None)
                            }
                            Err(e2) => {
                                ("unhealthy".to_string(), Some(e2))
                            }
                        }
                    }
                    Err(_) => {
                        ("pending_auth".to_string(), Some("OAuth token expired and refresh failed".to_string()))
                    }
                }
            } else {
                ("unhealthy".to_string(), Some(e))
            };
            
            sqlx::query(
                "UPDATE servers SET status = $1, last_health_check = NOW(), health_error = $2 WHERE id = $3"
            )
            .bind(&new_status)
            .bind(&error_msg)
            .bind(server.id)
            .execute(&db.pool)
            .await
            .map_err(|e| format!("Failed to update server status: {}", e))?;
            
            if new_status == "healthy" {
                Ok(())
            } else {
                Err(error_msg.unwrap_or_else(|| "Unknown error".to_string()))
            }
        }
    }
}

async fn get_access_token(db: &Database, server_id: Uuid, encryption_key: &str) -> Result<Option<String>, String> {
    // For health check, get any available token for this server (no user context)
    let row: Option<(String, Option<chrono::DateTime<Utc>>)> = sqlx::query_as(
        "SELECT access_token_encrypted, expires_at FROM oauth_tokens WHERE server_id = $1 LIMIT 1"
    )
    .bind(server_id)
    .fetch_optional(&db.pool)
    .await
    .map_err(|e| format!("Failed to fetch token: {}", e))?;
    
    if let Some((encrypted, expires_at)) = row {
        if let Some(exp) = expires_at {
            if exp < Utc::now() {
                return Ok(None);
            }
        }
        
        let key = crypto::derive_key(encryption_key);
        let decrypted = crypto::decrypt(&encrypted, &key)
            .map_err(|e| format!("Failed to decrypt token: {}", e))?;
        
        Ok(Some(decrypted))
    } else {
        Ok(None)
    }
}

async fn try_refresh_token(db: &Database, server_id: Uuid, encryption_key: &str) -> Result<String, String> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT refresh_token_encrypted FROM oauth_tokens WHERE server_id = $1 LIMIT 1"
    )
    .bind(server_id)
    .fetch_optional(&db.pool)
    .await
    .map_err(|e| format!("Failed to fetch refresh token: {}", e))?;
    
    let refresh_encrypted = row
        .and_then(|(r,)| r)
        .ok_or("No refresh token available")?;
    
    let key = crypto::derive_key(encryption_key);
    let _refresh_token = crypto::decrypt(&refresh_encrypted, &key)
        .map_err(|e| format!("Failed to decrypt refresh token: {}", e))?;
    
    Err("Token refresh not implemented yet - user must re-authorize".to_string())
}

async fn test_mcp_connection(server_url: &str, auth_header_name: Option<&str>, auth_header_value: Option<&str>) -> Result<(), String> {
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    
    // Build custom headers if auth is provided
    let mut custom_headers = HeaderMap::new();
    if let (Some(name), Some(value)) = (auth_header_name, auth_header_value) {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|e| format!("Invalid header name: {:?}", e))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|e| format!("Invalid header value: {:?}", e))?;
        custom_headers.insert(header_name, header_value);
    }

    // Build client with custom headers
    let client = if custom_headers.is_empty() {
        reqwest::Client::new()
    } else {
        reqwest::Client::builder()
            .default_headers(custom_headers)
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {:?}", e))?
    };

    let config = StreamableHttpClientTransportConfig::with_uri(server_url);
    let transport = StreamableHttpClientTransport::with_client(client, config);

    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "mcpx-healthcheck".to_string(),
            title: None,
            version: "1.0.0".to_string(),
            website_url: None,
            icons: None,
        },
    };

    let mcp_client: RunningService<rmcp::RoleClient, _> = client_info.serve(transport).await
        .map_err(|e| format!("Connection failed: {:?}", e))?;

    let _tools_result = mcp_client.list_tools(None).await
        .map_err(|e| format!("Failed to list tools: {:?}", e))?;

    let _ = mcp_client.cancel().await;

    Ok(())
}
