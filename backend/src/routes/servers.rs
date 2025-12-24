use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::messages::error;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::models::server::{Server, ServerResponse, CreateServerRequest, UpdateServerRequest};
use crate::routes::auth::{extract_token, validate_token};

use rmcp::{
    ServiceExt,
    model::{ClientCapabilities, ClientInfo, Implementation},
    transport::streamable_http_client::{StreamableHttpClientTransport, StreamableHttpClientTransportConfig},
    service::RunningService,
};

const SQL_LIST_SERVERS: &str = "SELECT * FROM servers WHERE user_id = $1 ORDER BY created_at DESC";

const SQL_INSERT_SERVER: &str = r#"
    INSERT INTO servers (user_id, name, url, transport, auth_type, status, oauth_client_id, oauth_authorization_url, oauth_token_url, oauth_scopes, oauth_use_pkce)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    RETURNING *
"#;

const SQL_INSERT_CREDENTIAL: &str = "INSERT INTO credentials (server_id, credential_type, encrypted_value, name) VALUES ($1, $2, $3, $4)";

const SQL_SELECT_SERVER_BY_NAME: &str = "SELECT * FROM servers WHERE user_id = $1 AND name = $2";

const SQL_UPDATE_SERVER: &str = r#"
    UPDATE servers SET
        name = COALESCE($3, name),
        url = COALESCE($4, url),
        transport = COALESCE($5, transport),
        enabled = COALESCE($6, enabled),
        updated_at = NOW()
    WHERE user_id = $1 AND name = $2
    RETURNING *
"#;

const SQL_DELETE_SERVER: &str = "DELETE FROM servers WHERE user_id = $1 AND name = $2";

const SQL_SELECT_OAUTH_TOKEN: &str = "SELECT access_token_encrypted FROM oauth_tokens WHERE server_id = $1 AND user_id = $2";

const SQL_UPDATE_SERVER_STATUS: &str = "UPDATE servers SET status = 'healthy', last_health_check = NOW(), health_error = NULL, updated_at = NOW() WHERE id = $1";

const SQL_UPDATE_SERVER_ERROR: &str = "UPDATE servers SET status = 'unhealthy', health_error = $1, updated_at = NOW() WHERE id = $2";

const ERR_FAILED_TO_STORE_CREDENTIAL: &str = "Failed to store credential";


async fn get_user_id(
    headers: &axum::http::HeaderMap,
    state: &AppState,
) -> Result<Uuid, (StatusCode, String)> {
    let token = extract_token(headers)?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;
    Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))
}

pub async fn list_servers(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<ServerResponse>>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let servers = sqlx::query_as::<_, Server>(SQL_LIST_SERVERS)
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    let base_url = &state.config.base_url;
    let responses: Vec<ServerResponse> = servers
        .into_iter()
        .map(|s| ServerResponse::from_server(s, user_id, base_url))
        .collect();

    Ok(Json(responses))
}

pub async fn create_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateServerRequest>,
) -> Result<(StatusCode, Json<ServerResponse>), (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;


    if !payload.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err((StatusCode::BAD_REQUEST, error::INVALID_SERVER_NAME.to_string()));
    }



    let initial_status = if payload.auth_type == "oauth_auto" {
        "pending_auth"
    } else {
        "pending_health"
    };

    let server = sqlx::query_as::<_, Server>(SQL_INSERT_SERVER)
    .bind(user_id)
    .bind(&payload.name)
    .bind(&payload.url)
    .bind(&payload.transport)
    .bind(&payload.auth_type)
    .bind(initial_status)
    .bind(&payload.oauth_client_id)
    .bind(&payload.oauth_authorization_url)
    .bind(&payload.oauth_token_url)
    .bind(&payload.oauth_scopes)
    .bind(if payload.auth_type.starts_with("oauth") { Some(true) } else { None })
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            (StatusCode::CONFLICT, error::SERVER_EXISTS.to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e))
        }
    })?;


    if let Some(ref api_key) = payload.api_key {
        if !api_key.is_empty() {
            let key = crate::services::crypto::derive_key(&state.config.encryption_key);
            let encrypted = crate::services::crypto::encrypt(api_key, &key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
            
            sqlx::query(SQL_INSERT_CREDENTIAL)
            .bind(server.id)
            .bind("api_key")
            .bind(&encrypted)
            .bind("API Key")
            .execute(&state.db.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", ERR_FAILED_TO_STORE_CREDENTIAL, e)))?;
        }
    }

    if let Some(ref bearer_token) = payload.bearer_token {
        if !bearer_token.is_empty() {
            let key = crate::services::crypto::derive_key(&state.config.encryption_key);
            let encrypted = crate::services::crypto::encrypt(bearer_token, &key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
            
            sqlx::query(SQL_INSERT_CREDENTIAL)
            .bind(server.id)
            .bind("bearer")
            .bind(&encrypted)
            .bind("Bearer Token")
            .execute(&state.db.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", ERR_FAILED_TO_STORE_CREDENTIAL, e)))?;
        }
    }

    if let Some(ref client_secret) = payload.oauth_client_secret {
        if !client_secret.is_empty() {
            let key = crate::services::crypto::derive_key(&state.config.encryption_key);
            let encrypted = crate::services::crypto::encrypt(client_secret, &key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
            
            sqlx::query(SQL_INSERT_CREDENTIAL)
            .bind(server.id)
            .bind("oauth_client_secret")
            .bind(&encrypted)
            .bind("OAuth Client Secret")
            .execute(&state.db.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", ERR_FAILED_TO_STORE_CREDENTIAL, e)))?;
        }
    }

    let base_url = &state.config.base_url;
    Ok((StatusCode::CREATED, Json(ServerResponse::from_server(server, user_id, base_url))))
}

pub async fn get_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<ServerResponse>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let server = sqlx::query_as::<_, Server>(SQL_SELECT_SERVER_BY_NAME)
    .bind(user_id)
    .bind(&name)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?
    .ok_or((StatusCode::NOT_FOUND, error::SERVER_NOT_FOUND.to_string()))?;

    let base_url = &state.config.base_url;
    Ok(Json(ServerResponse::from_server(server, user_id, base_url)))
}

pub async fn update_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<UpdateServerRequest>,
) -> Result<Json<ServerResponse>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;


    let server = sqlx::query_as::<_, Server>(SQL_UPDATE_SERVER)
    .bind(user_id)
    .bind(&name)
    .bind(&payload.name)
    .bind(&payload.url)
    .bind(&payload.transport)
    .bind(payload.enabled)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?
    .ok_or((StatusCode::NOT_FOUND, error::SERVER_NOT_FOUND.to_string()))?;

    let base_url = &state.config.base_url;
    Ok(Json(ServerResponse::from_server(server, user_id, base_url)))
}

pub async fn delete_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let result = sqlx::query(SQL_DELETE_SERVER)
        .bind(user_id)
        .bind(&name)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, error::SERVER_NOT_FOUND.to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

const SQL_SELECT_CREDENTIAL: &str = "SELECT encrypted_value FROM credentials WHERE server_id = $1 AND credential_type = $2";

pub async fn test_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<TestResult>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let server = sqlx::query_as::<_, Server>(SQL_SELECT_SERVER_BY_NAME)
    .bind(user_id)
    .bind(&name)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?
    .ok_or((StatusCode::NOT_FOUND, error::SERVER_NOT_FOUND.to_string()))?;

    // Fetch credentials based on auth_type
    let (auth_header_name, auth_header_value): (Option<String>, Option<String>) = 
        match server.auth_type.as_deref() {
            Some("api_key") => {
                tracing::info!("Server {} requires API Key auth", name);
                let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_CREDENTIAL)
                    .bind(server.id)
                    .bind("api_key")
                    .fetch_optional(&state.db.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;
                
                if let Some((encrypted,)) = row {
                    let key = crate::services::crypto::derive_key(&state.config.encryption_key);
                    match crate::services::crypto::decrypt(&encrypted, &key) {
                        Ok(api_key) => {
                            tracing::info!("API Key decrypted successfully");
                            (Some("X-API-Key".to_string()), Some(api_key))
                        }
                        Err(e) => {
                            tracing::error!("Failed to decrypt API Key: {:?}", e);
                            (None, None)
                        }
                    }
                } else {
                    tracing::warn!("No API Key found for server {}", name);
                    (None, None)
                }
            }
            Some("bearer") => {
                tracing::info!("Server {} requires Bearer Token auth", name);
                let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_CREDENTIAL)
                    .bind(server.id)
                    .bind("bearer")
                    .fetch_optional(&state.db.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;
                
                if let Some((encrypted,)) = row {
                    let key = crate::services::crypto::derive_key(&state.config.encryption_key);
                    match crate::services::crypto::decrypt(&encrypted, &key) {
                        Ok(token) => {
                            tracing::info!("Bearer Token decrypted successfully");
                            (Some("Authorization".to_string()), Some(format!("Bearer {}", token)))
                        }
                        Err(e) => {
                            tracing::error!("Failed to decrypt Bearer Token: {:?}", e);
                            (None, None)
                        }
                    }
                } else {
                    tracing::warn!("No Bearer Token found for server {}", name);
                    (None, None)
                }
            }
            Some("oauth_auto") => {
                tracing::info!("Server {} requires OAuth, fetching token", name);
                let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_OAUTH_TOKEN)
                    .bind(server.id)
                    .bind(user_id)
                    .fetch_optional(&state.db.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;
                
                if let Some((encrypted,)) = row {
                    let key = crate::services::crypto::derive_key(&state.config.encryption_key);
                    match crate::services::crypto::decrypt(&encrypted, &key) {
                        Ok(token) => {
                            tracing::info!("OAuth token decrypted successfully");
                            (Some("Authorization".to_string()), Some(format!("Bearer {}", token)))
                        }
                        Err(e) => {
                            tracing::error!("Failed to decrypt OAuth token: {:?}", e);
                            (None, None)
                        }
                    }
                } else {
                    tracing::warn!("No OAuth token found for server {}", name);
                    (None, None)
                }
            }
            Some("oauth_client_credentials") => {
                tracing::info!("Server {} requires OAuth Client Credentials", name);
                
                // Get client_id from server record
                let client_id = server.oauth_client_id.clone().unwrap_or_default();
                
                // Get client_secret from credentials
                let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_CREDENTIAL)
                    .bind(server.id)
                    .bind("oauth_client_secret")
                    .fetch_optional(&state.db.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;
                
                let client_secret = if let Some((encrypted,)) = row {
                    let key = crate::services::crypto::derive_key(&state.config.encryption_key);
                    crate::services::crypto::decrypt(&encrypted, &key).ok()
                } else {
                    None
                };
                
                // Get token URL from server record
                let token_url = server.oauth_token_url.clone().unwrap_or_default();
                
                if client_id.is_empty() || client_secret.is_none() || token_url.is_empty() {
                    tracing::warn!("Missing OAuth Client Credentials config for server {}", name);
                    (None, None)
                } else {
                    // Fetch token from token endpoint
                    let client = reqwest::Client::new();
                    let token_response = client.post(&token_url)
                        .json(&serde_json::json!({
                            "grant_type": "client_credentials",
                            "client_id": client_id,
                            "client_secret": client_secret.unwrap()
                        }))
                        .send()
                        .await;
                    
                    match token_response {
                        Ok(resp) if resp.status().is_success() => {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
                                    tracing::info!("OAuth Client Credentials token obtained successfully");
                                    (Some("Authorization".to_string()), Some(format!("Bearer {}", access_token)))
                                } else {
                                    tracing::error!("Token response missing access_token");
                                    (None, None)
                                }
                            } else {
                                tracing::error!("Failed to parse token response");
                                (None, None)
                            }
                        }
                        Ok(resp) => {
                            tracing::error!("Token endpoint returned error: {}", resp.status());
                            (None, None)
                        }
                        Err(e) => {
                            tracing::error!("Failed to fetch token: {:?}", e);
                            (None, None)
                        }
                    }
                }
            }
            _ => {
                tracing::info!("Server {} has no auth configured", name);
                (None, None)
            }
        };

    let start = std::time::Instant::now();

    let result = test_mcp_connection(&server.url, auth_header_name.as_deref(), auth_header_value.as_deref()).await;
    let latency_ms = start.elapsed().as_millis() as u32;
    
    match result {
        Ok(tools) => {
        
            let _ = sqlx::query(SQL_UPDATE_SERVER_STATUS)
                .bind(server.id)
                .execute(&state.db.pool)
                .await;
            
            Ok(Json(TestResult {
                success: true,
                message: "Connection successful".to_string(),
                latency_ms,
                status_code: Some(200),
                tools: Some(tools),
            }))
        }
        Err(e) => Ok(Json(TestResult {
            success: false,
            message: format!("Connection failed: {}", e),
            latency_ms,
            status_code: None,
            tools: None,
        })),
    }
}

async fn test_mcp_connection(server_url: &str, auth_header_name: Option<&str>, auth_header_value: Option<&str>) -> Result<Vec<ToolInfo>, String> {
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
            name: "mcpx-gateway".to_string(),
            title: None,
            version: "1.0.0".to_string(),
            website_url: None,
            icons: None,
        },
    };

    let client: RunningService<rmcp::RoleClient, _> = client_info.serve(transport).await
        .map_err(|e| format!("Failed to connect: {:?}", e))?;

    let tools_result = client.list_tools(None).await
        .map_err(|e| format!("Failed to list tools: {:?}", e))?;

    let _ = client.cancel().await;

    let tools: Vec<ToolInfo> = tools_result.tools.into_iter().map(|t| ToolInfo {
        name: t.name.to_string(),
        description: t.description.map(|d| d.into()),
    }).collect();

    Ok(tools)
}

#[derive(serde::Serialize)]
pub struct TestResult {
    success: bool,
    message: String,
    latency_ms: u32,
    status_code: Option<u16>,
    tools: Option<Vec<ToolInfo>>,
}

#[derive(serde::Serialize)]
pub struct ToolInfo {
    name: String,
    description: Option<String>,
}


