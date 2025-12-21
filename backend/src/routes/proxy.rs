use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    body::Body,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::services::crypto;

/// MCP Proxy handler (with user context - for future API key auth)
/// Route: POST /mcp/:user_id/:server_name
pub async fn mcp_proxy(
    State(state): State<Arc<AppState>>,
    Path((user_id, server_name)): Path<(String, String)>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, (StatusCode, String)> {
    tracing::info!("MCP Proxy request: user={}, server={}", user_id, server_name);
    
    // Parse user_id as UUID
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid user ID".to_string()))?;
    
    // Find server in database
    let server = get_server_by_name(&state, &server_name, user_uuid).await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Server not found: {}", e)))?;
    
    // Check server status
    if server.status.as_deref() == Some("disabled") {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "Server is disabled".to_string()));
    }
    
    // Build auth headers based on auth_type
    let auth_headers = build_auth_headers(&state, &server).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Auth error: {}", e)))?;
    
    // Forward request to real server
    let response = forward_request(&server.url, headers, auth_headers, body).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Proxy error: {}", e)))?;
    
    Ok(response)
}

#[derive(sqlx::FromRow)]
struct ServerRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
    url: String,
    transport: String,
    auth_type: Option<String>,
    status: Option<String>,
}

async fn get_server_by_name(state: &AppState, name: &str, user_id: Uuid) -> Result<ServerRow, String> {
    sqlx::query_as::<_, ServerRow>(
        "SELECT id, user_id, name, url, transport, auth_type, status FROM servers WHERE name = $1 AND user_id = $2"
    )
    .bind(name)
    .bind(user_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or_else(|| "Server not found".to_string())
}

async fn build_auth_headers(state: &AppState, server: &ServerRow) -> Result<Vec<(String, String)>, String> {
    let mut headers = Vec::new();
    
    match server.auth_type.as_deref() {
        Some("none") | None => {
            // No auth needed
        }
        Some("api_key") => {
            // Get API key from credentials
            if let Some(api_key) = get_credential(&state, server.id, "api_key").await? {
                headers.push(("X-API-Key".to_string(), api_key));
            }
        }
        Some("bearer") => {
            // Get bearer token from credentials
            if let Some(token) = get_credential(&state, server.id, "bearer_token").await? {
                headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
            }
        }
        Some("oauth_auto") => {
            // Get OAuth access token
            if let Some(access_token) = get_oauth_token(&state, server).await? {
                headers.push(("Authorization".to_string(), format!("Bearer {}", access_token)));
            } else {
                return Err("OAuth token not available - re-authorization needed".to_string());
            }
        }
        Some(other) => {
            tracing::warn!("Unknown auth type: {}", other);
        }
    }
    
    Ok(headers)
}

async fn get_credential(state: &AppState, server_id: Uuid, cred_type: &str) -> Result<Option<String>, String> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value_encrypted FROM credentials WHERE server_id = $1 AND key = $2"
    )
    .bind(server_id)
    .bind(cred_type)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| format!("Failed to fetch credential: {}", e))?;
    
    if let Some((encrypted,)) = row {
        // TODO: Decrypt credential
        // For now, assume it's stored plain (should use crypto::decrypt)
        Ok(Some(encrypted))
    } else {
        Ok(None)
    }
}

async fn get_oauth_token(state: &AppState, server: &ServerRow) -> Result<Option<String>, String> {
    let row: Option<(String, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT access_token_encrypted, expires_at FROM oauth_tokens WHERE server_id = $1 AND user_id = $2"
    )
    .bind(server.id)
    .bind(server.user_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| format!("Failed to fetch OAuth token: {}", e))?;
    
    if let Some((encrypted, expires_at)) = row {
        // Check if expired
        if let Some(exp) = expires_at {
            if exp < chrono::Utc::now() {
                return Err("OAuth token expired".to_string());
            }
        }
        
        // Decrypt token
        let key = crypto::derive_key(&state.config.encryption_key);
        let token = crypto::decrypt(&encrypted, &key)
            .map_err(|e| format!("Failed to decrypt token: {}", e))?;
        
        Ok(Some(token))
    } else {
        Ok(None)
    }
}

async fn forward_request(
    target_url: &str,
    original_headers: HeaderMap,
    auth_headers: Vec<(String, String)>,
    body: Body,
) -> Result<Response, String> {
    let client = reqwest::Client::new();
    
    // Build request
    let mut req_builder = client.post(target_url);
    
    // Copy relevant headers from original request
    if let Some(content_type) = original_headers.get("content-type") {
        req_builder = req_builder.header("Content-Type", content_type.to_str().unwrap_or("application/json"));
    } else {
        req_builder = req_builder.header("Content-Type", "application/json");
    }
    
    // MCP spec requires Accept header with both application/json and text/event-stream
    req_builder = req_builder.header("Accept", "application/json, text/event-stream");
    
    // Forward MCP-Session-Id header if present (required for session management)
    if let Some(session_id) = original_headers.get("mcp-session-id") {
        req_builder = req_builder.header("Mcp-Session-Id", session_id.to_str().unwrap_or(""));
    }
    
    // Add auth headers
    for (name, value) in auth_headers {
        req_builder = req_builder.header(&name, &value);
    }
    
    // Get body bytes
    let body_bytes = axum::body::to_bytes(body, 10 * 1024 * 1024) // 10MB limit
        .await
        .map_err(|e| format!("Failed to read body: {}", e))?;
    
    req_builder = req_builder.body(body_bytes.to_vec());
    
    // Send request
    let response = req_builder.send().await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    // Build response
    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = response.bytes().await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let mut builder = Response::builder().status(status.as_u16());
    
    // Copy response headers
    for (name, value) in headers.iter() {
        if name != "transfer-encoding" && name != "connection" {
            builder = builder.header(name.as_str(), value.to_str().unwrap_or(""));
        }
    }
    
    builder.body(Body::from(body_bytes.to_vec()))
        .map_err(|e| format!("Failed to build response: {}", e))
}
