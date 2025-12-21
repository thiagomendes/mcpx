use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::services::crypto;

// ============================================================================
// MCP Proxy Handler
// ============================================================================

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
    
    // Get governance config for this server
    let governance = get_governance_config(&state, server.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Governance error: {}", e)))?;
    
    // Build auth headers based on auth_type
    let auth_headers = build_auth_headers(&state, &server).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Auth error: {}", e)))?;
    
    // Read the request body to check the method
    let body_bytes = axum::body::to_bytes(body, 10 * 1024 * 1024)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read body: {}", e)))?;
    
    // Parse the JSON-RPC request to check method and potentially modify it
    let request: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;
    
    let method = request.get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    
    // Check if this is a tools/call request - need to validate and strip prefix
    let modified_body = if method == "tools/call" {
        handle_tools_call(&request, &governance)?
    } else {
        body_bytes.to_vec()
    };
    
    // Forward request to real server
    let response = forward_request(&server.url, headers, auth_headers, modified_body).await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Proxy error: {}", e)))?;
    
    // If this was a tools/list request, filter the response
    if method == "tools/list" {
        return filter_tools_response(response, &governance).await;
    }
    
    Ok(response)
}

// ============================================================================
// Governance Types
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct GovernanceConfig {
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
    pub tool_prefix: String,
}

impl GovernanceConfig {
    pub fn has_filter(&self) -> bool {
        !self.allowed_tools.is_empty() || !self.denied_tools.is_empty()
    }
    
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        // If whitelist is configured, tool must be in whitelist
        if !self.allowed_tools.is_empty() {
            return self.allowed_tools.iter().any(|t| t == tool_name);
        }
        
        // If blacklist is configured, tool must NOT be in blacklist
        if !self.denied_tools.is_empty() {
            return !self.denied_tools.iter().any(|t| t == tool_name);
        }
        
        // No filter configured - all tools allowed
        true
    }
    
    pub fn add_prefix(&self, tool_name: &str) -> String {
        if self.tool_prefix.is_empty() {
            tool_name.to_string()
        } else {
            format!("{}_{}", self.tool_prefix, tool_name)
        }
    }
    
    pub fn strip_prefix<'a>(&self, tool_name: &'a str) -> &'a str {
        if self.tool_prefix.is_empty() {
            return tool_name;
        }
        
        let prefix = format!("{}_", self.tool_prefix);
        tool_name.strip_prefix(&prefix).unwrap_or(tool_name)
    }
}

// ============================================================================
// Database Types
// ============================================================================

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

#[derive(sqlx::FromRow)]
struct GovernanceRow {
    allowed_tools: serde_json::Value,
    denied_tools: serde_json::Value,
    tool_prefix: String,
}

// ============================================================================
// MCP JSON-RPC Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct McpToolsListResponse {
    jsonrpc: String,
    id: serde_json::Value,
    result: Option<ToolsListResult>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolsListResult {
    tools: Vec<McpTool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McpTool {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "inputSchema", skip_serializing_if = "Option::is_none")]
    input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<serde_json::Value>,
}

// ============================================================================
// Database Functions
// ============================================================================

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

async fn get_governance_config(state: &AppState, server_id: Uuid) -> Result<GovernanceConfig, String> {
    let row: Option<GovernanceRow> = sqlx::query_as(
        "SELECT allowed_tools, denied_tools, tool_prefix FROM governance_configs WHERE server_id = $1"
    )
    .bind(server_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;
    
    match row {
        Some(r) => Ok(GovernanceConfig {
            allowed_tools: json_to_vec(&r.allowed_tools),
            denied_tools: json_to_vec(&r.denied_tools),
            tool_prefix: r.tool_prefix,
        }),
        None => Ok(GovernanceConfig::default()),
    }
}

fn json_to_vec(value: &serde_json::Value) -> Vec<String> {
    value.as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

// ============================================================================
// Governance Filtering
// ============================================================================

/// Handle tools/call request - validate tool allowed and strip prefix
fn handle_tools_call(request: &serde_json::Value, governance: &GovernanceConfig) -> Result<Vec<u8>, (StatusCode, String)> {
    // Get the tool name from params
    let tool_name = request
        .get("params")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");
    
    if tool_name.is_empty() {
        // No tool name - just forward as-is
        return serde_json::to_vec(request)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {}", e)));
    }
    
    // Strip prefix if configured
    let original_name = governance.strip_prefix(tool_name);
    
    // Check if tool is allowed
    if !governance.is_tool_allowed(original_name) {
        return Err((
            StatusCode::FORBIDDEN,
            format!("Tool '{}' is not allowed by governance policy", original_name),
        ));
    }
    
    // If prefix was stripped, modify the request
    if original_name != tool_name {
        let mut modified = request.clone();
        if let Some(params) = modified.get_mut("params") {
            if let Some(obj) = params.as_object_mut() {
                obj.insert("name".to_string(), serde_json::Value::String(original_name.to_string()));
            }
        }
        return serde_json::to_vec(&modified)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {}", e)));
    }
    
    serde_json::to_vec(request)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {}", e)))
}

/// Filter tools/list response based on governance config
async fn filter_tools_response(response: Response, governance: &GovernanceConfig) -> Result<Response, (StatusCode, String)> {
    // If no governance configured, return as-is
    if !governance.has_filter() && governance.tool_prefix.is_empty() {
        return Ok(response);
    }
    
    let status = response.status();
    let headers = response.headers().clone();
    
    // Read body
    let body_bytes = axum::body::to_bytes(response.into_body(), 10 * 1024 * 1024)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read response: {}", e)))?;
    
    // Try parsing as SSE (event: message\ndata: {...})
    let body_str = String::from_utf8_lossy(&body_bytes);
    
    // Check if it's SSE format
    if body_str.contains("event:") && body_str.contains("data:") {
        // Parse SSE and filter each data line
        let filtered = filter_sse_response(&body_str, governance);
        
        let mut builder = Response::builder().status(status);
        for (name, value) in headers.iter() {
            if name != "transfer-encoding" && name != "connection" && name != "content-length" {
                builder = builder.header(name.as_str(), value.to_str().unwrap_or(""));
            }
        }
        
        return builder.body(Body::from(filtered))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to build response: {}", e)));
    }
    
    // Try parsing as plain JSON-RPC
    if let Ok(mut json_response) = serde_json::from_slice::<McpToolsListResponse>(&body_bytes) {
        if let Some(ref mut result) = json_response.result {
            result.tools = filter_tools(&result.tools, governance);
        }
        
        let filtered_body = serde_json::to_vec(&json_response)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {}", e)))?;
        
        let mut builder = Response::builder().status(status);
        for (name, value) in headers.iter() {
            if name != "transfer-encoding" && name != "connection" && name != "content-length" {
                builder = builder.header(name.as_str(), value.to_str().unwrap_or(""));
            }
        }
        
        return builder.body(Body::from(filtered_body))
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to build response: {}", e)));
    }
    
    // Can't parse - return as-is
    let mut builder = Response::builder().status(status);
    for (name, value) in headers.iter() {
        if name != "transfer-encoding" && name != "connection" {
            builder = builder.header(name.as_str(), value.to_str().unwrap_or(""));
        }
    }
    
    builder.body(Body::from(body_bytes.to_vec()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to build response: {}", e)))
}

/// Filter SSE response containing tools/list
fn filter_sse_response(body: &str, governance: &GovernanceConfig) -> String {
    let mut result = String::new();
    let mut current_event = String::new();
    let mut current_data = String::new();
    
    for line in body.lines() {
        if line.starts_with("event:") {
            current_event = line.to_string();
        } else if line.starts_with("data:") {
            current_data = line[5..].trim().to_string();
            
            // Try to parse and filter JSON-RPC response
            if let Ok(mut json_response) = serde_json::from_str::<McpToolsListResponse>(&current_data) {
                if let Some(ref mut res) = json_response.result {
                    res.tools = filter_tools(&res.tools, governance);
                }
                
                if let Ok(filtered_json) = serde_json::to_string(&json_response) {
                    result.push_str(&current_event);
                    result.push('\n');
                    result.push_str("data: ");
                    result.push_str(&filtered_json);
                    result.push('\n');
                    continue;
                }
            }
            
            // If parsing failed, keep original
            result.push_str(&current_event);
            result.push('\n');
            result.push_str("data: ");
            result.push_str(&current_data);
            result.push('\n');
        } else if line.is_empty() {
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    
    result
}

/// Filter and transform tools list
fn filter_tools(tools: &[McpTool], governance: &GovernanceConfig) -> Vec<McpTool> {
    tools.iter()
        .filter(|t| governance.is_tool_allowed(&t.name))
        .map(|t| {
            let mut tool = t.clone();
            tool.name = governance.add_prefix(&t.name);
            tool
        })
        .collect()
}

// ============================================================================
// Auth & Credentials
// ============================================================================

async fn build_auth_headers(state: &AppState, server: &ServerRow) -> Result<Vec<(String, String)>, String> {
    let mut headers = Vec::new();
    
    match server.auth_type.as_deref() {
        Some("none") | None => {
            // No auth needed
        }
        Some("api_key") => {
            if let Some(api_key) = get_credential(state, server.id, "api_key").await? {
                headers.push(("X-API-Key".to_string(), api_key));
            }
        }
        Some("bearer") => {
            if let Some(token) = get_credential(state, server.id, "bearer_token").await? {
                headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
            }
        }
        Some("oauth_auto") => {
            if let Some(access_token) = get_oauth_token(state, server).await? {
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
        if let Some(exp) = expires_at {
            if exp < chrono::Utc::now() {
                return Err("OAuth token expired".to_string());
            }
        }
        
        let key = crypto::derive_key(&state.config.encryption_key);
        let token = crypto::decrypt(&encrypted, &key)
            .map_err(|e| format!("Failed to decrypt token: {}", e))?;
        
        Ok(Some(token))
    } else {
        Ok(None)
    }
}

// ============================================================================
// Request Forwarding
// ============================================================================

async fn forward_request(
    target_url: &str,
    original_headers: HeaderMap,
    auth_headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> Result<Response, String> {
    let client = reqwest::Client::new();
    let mut req_builder = client.post(target_url);
    
    // Set Content-Type
    if let Some(content_type) = original_headers.get("content-type") {
        req_builder = req_builder.header("Content-Type", content_type.to_str().unwrap_or("application/json"));
    } else {
        req_builder = req_builder.header("Content-Type", "application/json");
    }
    
    // MCP spec requires Accept header
    req_builder = req_builder.header("Accept", "application/json, text/event-stream");
    
    // Forward MCP-Session-Id if present
    if let Some(session_id) = original_headers.get("mcp-session-id") {
        req_builder = req_builder.header("Mcp-Session-Id", session_id.to_str().unwrap_or(""));
    }
    
    // Add auth headers
    for (name, value) in auth_headers {
        req_builder = req_builder.header(&name, &value);
    }
    
    req_builder = req_builder.body(body);
    
    // Send request
    let response = req_builder.send().await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    // Build response
    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = response.bytes().await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    let mut builder = Response::builder().status(status.as_u16());
    
    for (name, value) in headers.iter() {
        if name != "transfer-encoding" && name != "connection" {
            builder = builder.header(name.as_str(), value.to_str().unwrap_or(""));
        }
    }
    
    builder.body(Body::from(body_bytes.to_vec()))
        .map_err(|e| format!("Failed to build response: {}", e))
}
