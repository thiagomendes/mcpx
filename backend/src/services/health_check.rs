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

#[derive(Debug, sqlx::FromRow)]
struct ServerForHealthCheck {
    id: Uuid,
    user_id: Uuid,
    url: String,
    auth_type: Option<String>,
    status: Option<String>,
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
        "SELECT id, user_id, url, auth_type, status FROM servers WHERE status != 'pending_auth' AND enabled = true"
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

    let access_token: Option<String> = if server.auth_type.as_deref() == Some("oauth_auto") {
        get_access_token(db, server.id, server.user_id, encryption_key).await?
    } else {
        None
    };
    

    let result = test_mcp_connection(&server.url, access_token.as_deref()).await;
    
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
            
                match try_refresh_token(db, server.id, server.user_id, encryption_key).await {
                    Ok(new_token) => {
                    
                        match test_mcp_connection(&server.url, Some(&new_token)).await {
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

async fn get_access_token(db: &Database, server_id: Uuid, user_id: Uuid, encryption_key: &str) -> Result<Option<String>, String> {
    let row: Option<(String, Option<chrono::DateTime<Utc>>)> = sqlx::query_as(
        "SELECT access_token_encrypted, expires_at FROM oauth_tokens WHERE server_id = $1 AND user_id = $2"
    )
    .bind(server_id)
    .bind(user_id)
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

async fn try_refresh_token(db: &Database, server_id: Uuid, user_id: Uuid, encryption_key: &str) -> Result<String, String> {

    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT refresh_token_encrypted FROM oauth_tokens WHERE server_id = $1 AND user_id = $2"
    )
    .bind(server_id)
    .bind(user_id)
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

async fn test_mcp_connection(server_url: &str, access_token: Option<&str>) -> Result<(), String> {

    let config = if let Some(token) = access_token {
        StreamableHttpClientTransportConfig::with_uri(server_url)
            .auth_header(token)
    } else {
        StreamableHttpClientTransportConfig::with_uri(server_url)
    };

    let transport = StreamableHttpClientTransport::with_client(reqwest::Client::new(), config);


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


    let client: RunningService<rmcp::RoleClient, _> = client_info.serve(transport).await
        .map_err(|e| format!("Connection failed: {:?}", e))?;


    let _tools_result = client.list_tools(None).await
        .map_err(|e| format!("Failed to list tools: {:?}", e))?;


    let _ = client.cancel().await;

    Ok(())
}

