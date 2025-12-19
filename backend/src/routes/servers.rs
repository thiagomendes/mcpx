use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::models::server::{Server, ServerResponse, CreateServerRequest, UpdateServerRequest};
use crate::routes::auth::{extract_token, validate_token};

// rmcp SDK imports for MCP client
use rmcp::{
    ServiceExt,
    model::{ClientCapabilities, ClientInfo, Implementation},
    transport::streamable_http_client::{StreamableHttpClientTransport, StreamableHttpClientTransportConfig},
    service::RunningService,
};

// Helper: Get user_id from headers
async fn get_user_id(
    headers: &axum::http::HeaderMap,
    state: &AppState,
) -> Result<Uuid, (StatusCode, String)> {
    let token = extract_token(headers)?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;
    Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))
}

// GET /api/servers - List all servers for current user
pub async fn list_servers(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<ServerResponse>>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let servers = sqlx::query_as::<_, Server>(
        "SELECT * FROM servers WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let base_url = &state.config.base_url;
    let responses: Vec<ServerResponse> = servers
        .into_iter()
        .map(|s| ServerResponse::from_server(s, user_id, base_url))
        .collect();

    Ok(Json(responses))
}

// POST /api/servers - Create new server
pub async fn create_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateServerRequest>,
) -> Result<(StatusCode, Json<ServerResponse>), (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    // Validate name (alphanumeric, hyphens, underscores)
    if !payload.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err((StatusCode::BAD_REQUEST, "Invalid server name. Use only letters, numbers, hyphens, and underscores.".to_string()));
    }

    // Insert server with auth fields
    // Set initial status: pending_auth for oauth_auto, pending_health for others (awaits first health check)
    let initial_status = if payload.auth_type == "oauth_auto" {
        "pending_auth"
    } else {
        "pending_health"
    };

    let server = sqlx::query_as::<_, Server>(
        r#"
        INSERT INTO servers (user_id, name, url, transport, auth_type, status, oauth_client_id, oauth_authorization_url, oauth_token_url, oauth_scopes, oauth_use_pkce)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
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
            (StatusCode::CONFLICT, "Server with this name already exists".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
        }
    })?;

    // Store encrypted credentials if provided
    if let Some(ref api_key) = payload.api_key {
        if !api_key.is_empty() {
            let key = crate::services::crypto::derive_key(&state.config.encryption_key);
            let encrypted = crate::services::crypto::encrypt(api_key, &key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
            
            sqlx::query(
                "INSERT INTO credentials (server_id, credential_type, encrypted_value, name) VALUES ($1, 'api_key', $2, 'API Key')"
            )
            .bind(server.id)
            .bind(&encrypted)
            .execute(&state.db.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to store credential: {}", e)))?;
        }
    }

    if let Some(ref bearer_token) = payload.bearer_token {
        if !bearer_token.is_empty() {
            let key = crate::services::crypto::derive_key(&state.config.encryption_key);
            let encrypted = crate::services::crypto::encrypt(bearer_token, &key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
            
            sqlx::query(
                "INSERT INTO credentials (server_id, credential_type, encrypted_value, name) VALUES ($1, 'bearer', $2, 'Bearer Token')"
            )
            .bind(server.id)
            .bind(&encrypted)
            .execute(&state.db.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to store credential: {}", e)))?;
        }
    }

    if let Some(ref client_secret) = payload.oauth_client_secret {
        if !client_secret.is_empty() {
            let key = crate::services::crypto::derive_key(&state.config.encryption_key);
            let encrypted = crate::services::crypto::encrypt(client_secret, &key)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
            
            sqlx::query(
                "INSERT INTO credentials (server_id, credential_type, encrypted_value, name) VALUES ($1, 'oauth_client_secret', $2, 'OAuth Client Secret')"
            )
            .bind(server.id)
            .bind(&encrypted)
            .execute(&state.db.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to store credential: {}", e)))?;
        }
    }

    let base_url = &state.config.base_url;
    Ok((StatusCode::CREATED, Json(ServerResponse::from_server(server, user_id, base_url))))
}

// GET /api/servers/:name - Get server by name
pub async fn get_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<ServerResponse>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let server = sqlx::query_as::<_, Server>(
        "SELECT * FROM servers WHERE user_id = $1 AND name = $2"
    )
    .bind(user_id)
    .bind(&name)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
    .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))?;

    let base_url = &state.config.base_url;
    Ok(Json(ServerResponse::from_server(server, user_id, base_url)))
}

// PUT /api/servers/:name - Update server
pub async fn update_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<UpdateServerRequest>,
) -> Result<Json<ServerResponse>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    // Build dynamic update query
    let server = sqlx::query_as::<_, Server>(
        r#"
        UPDATE servers SET
            name = COALESCE($3, name),
            url = COALESCE($4, url),
            transport = COALESCE($5, transport),
            enabled = COALESCE($6, enabled),
            updated_at = NOW()
        WHERE user_id = $1 AND name = $2
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(&name)
    .bind(&payload.name)
    .bind(&payload.url)
    .bind(&payload.transport)
    .bind(&payload.enabled)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
    .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))?;

    let base_url = &state.config.base_url;
    Ok(Json(ServerResponse::from_server(server, user_id, base_url)))
}

// DELETE /api/servers/:name - Delete server
pub async fn delete_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let result = sqlx::query("DELETE FROM servers WHERE user_id = $1 AND name = $2")
        .bind(user_id)
        .bind(&name)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Server not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// POST /api/servers/:name/test - Test server connectivity using rmcp SDK
pub async fn test_server(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<TestResult>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let server = sqlx::query_as::<_, Server>(
        "SELECT * FROM servers WHERE user_id = $1 AND name = $2"
    )
    .bind(user_id)
    .bind(&name)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
    .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))?;

    // Get OAuth token if this is an oauth_auto server
    let access_token: Option<String> = if server.auth_type.as_deref() == Some("oauth_auto") {
        tracing::info!("Server {} requires OAuth, fetching token", name);
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT access_token_encrypted FROM oauth_tokens WHERE server_id = $1 AND user_id = $2"
        )
        .bind(&server.id)
        .bind(&user_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

        if let Some((encrypted,)) = row {
            tracing::info!("Found encrypted token, decrypting...");
            let key = crate::services::crypto::derive_key(&state.config.encryption_key);
            let decrypted = crate::services::crypto::decrypt(&encrypted, &key);
            match decrypted {
                Ok(token) => {
                    tracing::info!("Token decrypted successfully, length: {}", token.len());
                    Some(token)
                }
                Err(e) => {
                    tracing::error!("Failed to decrypt token: {:?}", e);
                    None
                }
            }
        } else {
            tracing::warn!("No token found in database for server {}", name);
            None
        }
    } else {
        None
    };

    let start = std::time::Instant::now();
    
    // Use rmcp SDK for proper MCP protocol handling
    let result = test_mcp_connection(&server.url, access_token.as_deref()).await;
    let latency_ms = start.elapsed().as_millis() as u32;
    
    match result {
        Ok(tools) => {
            // Update server status to healthy and record successful health check
            let _ = sqlx::query("UPDATE servers SET status = 'healthy', last_health_check = NOW(), health_error = NULL, updated_at = NOW() WHERE id = $1")
                .bind(&server.id)
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

/// Test MCP connection using rmcp SDK and return list of tools
async fn test_mcp_connection(server_url: &str, access_token: Option<&str>) -> Result<Vec<ToolInfo>, String> {
    // Build transport config with optional auth header
    let config = if let Some(token) = access_token {
        StreamableHttpClientTransportConfig::with_uri(server_url)
            .auth_header(token)
    } else {
        StreamableHttpClientTransportConfig::with_uri(server_url)
    };

    let transport = StreamableHttpClientTransport::with_client(reqwest::Client::new(), config);

    // Create client info
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

    // Connect and initialize - explicit type for peer info
    let client: RunningService<rmcp::RoleClient, _> = client_info.serve(transport).await
        .map_err(|e| format!("Failed to connect: {:?}", e))?;

    // List tools
    let tools_result = client.list_tools(None).await
        .map_err(|e| format!("Failed to list tools: {:?}", e))?;

    // Cancel/cleanup the client
    let _ = client.cancel().await;

    // Convert to our ToolInfo type
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


