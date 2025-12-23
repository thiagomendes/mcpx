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
use crate::services::mcp_client;
use crate::messages::error;


const SQL_SELECT_SERVER: &str = "SELECT id, user_id, name, url, transport, auth_type, status FROM servers WHERE name = $1 AND user_id = $2";
const SQL_SELECT_GOVERNANCE: &str = "SELECT allowed_tools, denied_tools, tool_prefix FROM governance_configs WHERE server_id = $1";

const SQL_SELECT_GATEWAY: &str = r#"
    SELECT g.id, g.name, g.slug FROM gateways g WHERE g.slug = $1 AND g.user_id = $2 AND g.enabled = true
"#;

const SQL_SELECT_GATEWAY_SERVERS: &str = r#"
    SELECT s.id, s.user_id, s.name, s.url, s.transport, s.auth_type, s.status
    FROM gateway_servers gs
    JOIN servers s ON s.id = gs.server_id
    WHERE gs.gateway_id = $1
    ORDER BY gs.priority
"#;

const SQL_UPSERT_GATEWAY_SESSION: &str = r#"
    INSERT INTO gateway_sessions (id, gateway_id, user_id, server_sessions, expires_at)
    VALUES ($1, $2, $3, $4, NOW() + INTERVAL '1 hour')
    ON CONFLICT (id) DO UPDATE SET server_sessions = $4, expires_at = NOW() + INTERVAL '1 hour'
"#;

const SQL_SELECT_GATEWAY_SESSION: &str = "SELECT server_sessions FROM gateway_sessions WHERE id = $1";

pub async fn mcp_proxy(
    State(state): State<Arc<AppState>>,
    Path((user_id, server_name)): Path<(String, String)>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, (StatusCode, String)> {
    tracing::info!("MCP Proxy request: user={}, target={}", user_id, server_name);
    
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, error::INVALID_USER_ID.to_string()))?;
    
    if let Ok(server) = get_server_by_name(&state, &server_name, user_uuid).await {
        return handle_server_proxy(&state, server, headers, body).await;
    }
    
    if let Ok(gateway) = get_gateway_by_slug(&state, &server_name, user_uuid).await {
        return handle_gateway_proxy(&state, gateway, user_uuid, headers, body).await;
    }
    
    Err((StatusCode::NOT_FOUND, format!("{}: no server or gateway found with name '{}'", error::SERVER_NOT_FOUND, server_name)))
}

async fn handle_server_proxy(
    state: &AppState,
    server: ServerRow,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, (StatusCode, String)> {
    let start = std::time::Instant::now();

    if server.status.as_deref() == Some("disabled") {
        return Err((StatusCode::SERVICE_UNAVAILABLE, error::SERVER_DISABLED.to_string()));
    }
    
    let governance = get_governance_config(state, server.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::GOVERNANCE_ERROR, e)))?;
    
    let auth_headers = build_auth_headers(state, &server).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::AUTH_ERROR, e)))?;
    
    let body_bytes = axum::body::to_bytes(body, 10 * 1024 * 1024)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("{}: {}", error::FAILED_TO_READ_BODY, e)))?;
    
    let request: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("{}: {}", error::INVALID_JSON, e)))?;
    
    let method = request.get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    
    // Extract tool_name when method is tools/call
    let tool_name = if method == "tools/call" {
        request.get("params")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };
    
    let modified_body = if method == "tools/call" {
        handle_tools_call(&request, &governance)?
    } else {
        body_bytes.to_vec()
    };
    
    let result = forward_request(&server.url, headers, auth_headers, modified_body).await;
    
    // Record metrics
    let latency_ms = start.elapsed().as_millis() as i32;
    let success = result.is_ok();
    let _ = crate::services::metrics::record_request(
        &state.db.pool,
        server.user_id,
        "server",
        server.id,
        &server.name,
        Some(method),
        tool_name.as_deref(),
        latency_ms,
        success,
    ).await;

    // Handle result and capture response body for audit
    match result {
        Ok(response) => {
            let status_code = response.status().as_u16() as i32;
            
            // Read response body bytes to capture for audit
            let (parts, body) = response.into_parts();
            let body_bytes = axum::body::to_bytes(body, 10 * 1024 * 1024)
                .await
                .unwrap_or_default();
            
            // Parse response body as JSON for audit log
            // First try direct JSON, then try extracting from SSE format (data: {...})
            let response_body_json: Option<serde_json::Value> = serde_json::from_slice(&body_bytes).ok()
                .or_else(|| {
                    // Try to parse as SSE - extract JSON from "data: {...}" lines
                    let body_str = std::str::from_utf8(&body_bytes).ok()?;
                    for line in body_str.lines() {
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            if let Ok(json) = serde_json::from_str(json_str) {
                                return Some(json);
                            }
                        }
                    }
                    None
                });
            
            // Check if JSON-RPC response contains an error
            let has_jsonrpc_error = response_body_json
                .as_ref()
                .and_then(|v| v.get("error"))
                .map(|e| !e.is_null())
                .unwrap_or(false);
            
            let actual_success = status_code >= 200 && status_code < 300 && !has_jsonrpc_error;
            
            // Extract error message if present
            let error_message = if has_jsonrpc_error {
                response_body_json
                    .as_ref()
                    .and_then(|v| v.get("error"))
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            };
            
            // Record audit log with full request/response
            let _ = crate::services::audit::record_audit_log(
                &state.db.pool,
                server.user_id,
                "server",
                server.id,
                &server.name,
                Some(method),
                tool_name.as_deref(),
                Some(request.clone()),
                response_body_json,
                status_code,
                error_message.as_deref(),
                latency_ms,
                actual_success,
                None,
            ).await;
            
            // Reconstruct response with the same body
            let response = Response::from_parts(parts, Body::from(body_bytes));
            
            if method == "tools/list" {
                return filter_tools_response(response, &governance).await;
            }
            
            Ok(response)
        }
        Err(msg) => {
            // Record audit log for error case
            let _ = crate::services::audit::record_audit_log(
                &state.db.pool,
                server.user_id,
                "server",
                server.id,
                &server.name,
                Some(method),
                tool_name.as_deref(),
                Some(request.clone()),
                None,
                502,
                Some(&msg),
                latency_ms,
                false,
                None,
            ).await;
            
            Err((StatusCode::BAD_GATEWAY, format!("{}: {}", error::PROXY_ERROR, msg)))
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct GatewayRow {
    id: Uuid,
    name: String,
    slug: String,
}

async fn get_gateway_by_slug(state: &AppState, slug: &str, user_id: Uuid) -> Result<GatewayRow, String> {
    sqlx::query_as::<_, GatewayRow>(SQL_SELECT_GATEWAY)
        .bind(slug)
        .bind(user_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Gateway not found".to_string())
}

async fn get_gateway_servers(state: &AppState, gateway_id: Uuid) -> Result<Vec<ServerRow>, String> {
    sqlx::query_as::<_, ServerRow>(SQL_SELECT_GATEWAY_SERVERS)
        .bind(gateway_id)
        .fetch_all(&state.db.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))
}

async fn handle_gateway_proxy(
    state: &AppState,
    gateway: GatewayRow,
    user_id: Uuid,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, (StatusCode, String)> {
    let start = std::time::Instant::now();
    tracing::info!("Gateway proxy: {}", gateway.name);
    
    let servers = get_gateway_servers(state, gateway.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get gateway servers: {}", e)))?;
    
    if servers.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Gateway has no servers".to_string()));
    }
    
    let body_bytes = axum::body::to_bytes(body, 10 * 1024 * 1024)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("{}: {}", error::FAILED_TO_READ_BODY, e)))?;
    
    let request: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("{}: {}", error::INVALID_JSON, e)))?;
    
    let method = request.get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    
    // Extract tool_name when method is tools/call
    let tool_name = if method == "tools/call" {
        request.get("params")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    } else {
        None
    };
    
    let result = match method {
        "initialize" => handle_gateway_initialize(state, &gateway, user_id, &servers, headers, body_bytes.to_vec()).await,
        "tools/list" => handle_gateway_tools_list(state, &gateway, &servers, headers, &request, body_bytes.to_vec()).await,
        "tools/call" => handle_gateway_tools_call(state, &gateway, &servers, headers, &request, body_bytes.to_vec()).await,
        _ => {
            if let Some(server) = servers.first() {
                let auth_headers = build_auth_headers(state, server).await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::AUTH_ERROR, e)))?;
                forward_request(&server.url, headers, auth_headers, body_bytes.to_vec()).await
                    .map_err(|e| (StatusCode::BAD_GATEWAY, format!("{}: {}", error::PROXY_ERROR, e)))
            } else {
                Err((StatusCode::BAD_REQUEST, "No servers in gateway".to_string()))
            }
        }
    };

    // Record gateway metrics
    let latency_ms = start.elapsed().as_millis() as i32;
    let success = result.is_ok();
    let _ = crate::services::metrics::record_request(
        &state.db.pool,
        user_id,
        "gateway",
        gateway.id,
        &gateway.name,
        Some(method),
        tool_name.as_deref(),
        latency_ms,
        success,
    ).await;

    // Handle result and capture response body for audit
    match result {
        Ok(response) => {
            let status_code = response.status().as_u16() as i32;
            
            // Read response body bytes to capture for audit
            let (parts, body) = response.into_parts();
            let body_bytes = axum::body::to_bytes(body, 10 * 1024 * 1024)
                .await
                .unwrap_or_default();
            
            // Parse response body as JSON for audit log
            // First try direct JSON, then try extracting from SSE format (data: {...})
            let response_body_json: Option<serde_json::Value> = serde_json::from_slice(&body_bytes).ok()
                .or_else(|| {
                    // Try to parse as SSE - extract JSON from "data: {...}" lines
                    let body_str = std::str::from_utf8(&body_bytes).ok()?;
                    for line in body_str.lines() {
                        if let Some(json_str) = line.strip_prefix("data: ") {
                            if let Ok(json) = serde_json::from_str(json_str) {
                                return Some(json);
                            }
                        }
                    }
                    None
                });
            
            // Check if JSON-RPC response contains an error
            let has_jsonrpc_error = response_body_json
                .as_ref()
                .and_then(|v| v.get("error"))
                .map(|e| !e.is_null())
                .unwrap_or(false);
            
            let actual_success = status_code >= 200 && status_code < 300 && !has_jsonrpc_error;
            
            // Extract error message if present
            let error_message = if has_jsonrpc_error {
                response_body_json
                    .as_ref()
                    .and_then(|v| v.get("error"))
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            };
            
            // Record audit log with full request/response
            let _ = crate::services::audit::record_audit_log(
                &state.db.pool,
                user_id,
                "gateway",
                gateway.id,
                &gateway.name,
                Some(method),
                tool_name.as_deref(),
                Some(request.clone()),
                response_body_json,
                status_code,
                error_message.as_deref(),
                latency_ms,
                actual_success,
                None,
            ).await;
            
            // Reconstruct response with the same body
            Ok(Response::from_parts(parts, Body::from(body_bytes)))
        }
        Err((status, msg)) => {
            // Record audit log for error case
            let _ = crate::services::audit::record_audit_log(
                &state.db.pool,
                user_id,
                "gateway",
                gateway.id,
                &gateway.name,
                Some(method),
                tool_name.as_deref(),
                Some(request.clone()),
                None,
                status.as_u16() as i32,
                Some(&msg),
                latency_ms,
                false,
                None,
            ).await;
            
            Err((status, msg))
        }
    }
}

async fn handle_gateway_initialize(
    state: &AppState,
    gateway: &GatewayRow,
    user_id: Uuid,
    servers: &[ServerRow],
    headers: HeaderMap,
    body: Vec<u8>,
) -> Result<Response, (StatusCode, String)> {
    let gateway_session_id = format!("gw_{}", Uuid::new_v4().to_string().replace("-", ""));
    let mut server_sessions: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut last_response: Option<Response> = None;
    
    for server in servers {
        let auth_headers = build_auth_headers(state, server).await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::AUTH_ERROR, e)))?;
        
        let response = forward_request(&server.url, headers.clone(), auth_headers, body.clone()).await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to initialize {}: {}", server.name, e)))?;
        
        let session_id = response.headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        
        server_sessions.insert(server.name.clone(), session_id);
        last_response = Some(response);
    }
    
    let sessions_json = serde_json::to_value(&server_sessions)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {}", e)))?;
    
    sqlx::query(SQL_UPSERT_GATEWAY_SESSION)
        .bind(&gateway_session_id)
        .bind(gateway.id)
        .bind(user_id)
        .bind(&sessions_json)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save session: {}", e)))?;
    
    let mut response = last_response.unwrap_or_else(|| {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap()
    });
    
    response.headers_mut().insert(
        "mcp-session-id",
        gateway_session_id.parse().unwrap(),
    );
    
    Ok(response)
}

async fn handle_gateway_tools_list(
    state: &AppState,
    _gateway: &GatewayRow,
    servers: &[ServerRow],
    _headers: HeaderMap,
    request: &serde_json::Value,
    _body: Vec<u8>,
) -> Result<Response, (StatusCode, String)> {
    #[derive(Debug, Clone, Serialize)]
    struct AggregatedTool {
        name: String,
        description: Option<String>,
        #[serde(rename = "inputSchema")]
        input_schema: serde_json::Value,
    }
    
    let mut all_tools: Vec<AggregatedTool> = Vec::new();
    
    for server in servers {
        let governance = get_governance_config(state, server.id).await
            .unwrap_or_default();
        
        let access_token = get_oauth_token(state, server).await.ok().flatten();
        
        let tools = mcp_client::list_tools_from_server(&server.url, access_token.as_deref()).await;
        
        match tools {
            Ok(tools) => {
                for tool in tools {
                    let original_name = tool.name.clone();
                    
                    if !governance.is_tool_allowed(&original_name) {
                        continue;
                    }
                    
                    // Only use governance prefix, no server name prefix
                    let final_name = governance.add_prefix(&original_name);
                    
                    all_tools.push(AggregatedTool {
                        name: final_name,
                        description: tool.description,
                        input_schema: tool.input_schema,
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Failed to list tools from {}: {}", server.name, e);
            }
        }
    }
    
    let request_id = request.get("id").cloned().unwrap_or(serde_json::json!(1));
    
    let response_json = serde_json::json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "tools": all_tools
        }
    });
    
    let sse_body = format!("data: {}\n\n", response_json);
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream")
        .body(Body::from(sse_body))
        .unwrap())
}


async fn handle_gateway_tools_call(
    state: &AppState,
    _gateway: &GatewayRow,
    servers: &[ServerRow],
    headers: HeaderMap,
    request: &serde_json::Value,
    _body: Vec<u8>,
) -> Result<Response, (StatusCode, String)> {
    let tool_name = request
        .get("params")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or((StatusCode::BAD_REQUEST, "Missing tool name".to_string()))?;
    
    tracing::info!("Gateway tools/call: looking for tool '{}'", tool_name);
    
    // Try each server to find one that has this tool
    for server in servers {
        let governance = get_governance_config(state, server.id).await.unwrap_or_default();
        
        tracing::debug!(
            "Server '{}': prefix='{}', auth_type={:?}", 
            server.name, 
            governance.tool_prefix,
            server.auth_type
        );
        
        // Strip governance prefix to get original tool name
        let original_tool_name = governance.strip_prefix(tool_name);
        
        // Check if this tool is allowed by governance
        if !governance.is_tool_allowed(original_tool_name) {
            tracing::debug!("Tool '{}' not allowed by governance", original_tool_name);
            continue;
        }
        
        // Check if this was the right server (prefix matched)
        let expected_prefixed = governance.add_prefix(original_tool_name);
        if expected_prefixed != tool_name {
            tracing::debug!(
                "Prefix mismatch: expected '{}' but got '{}'", 
                expected_prefixed, 
                tool_name
            );
            continue;
        }
        
        tracing::info!(
            "Routing tool '{}' to server '{}' (original: '{}')", 
            tool_name, 
            server.name, 
            original_tool_name
        );
        
        // Found the right server - forward the request
        let mut modified_request = request.clone();
        if let Some(params) = modified_request.get_mut("params") {
            if let Some(name) = params.get_mut("name") {
                *name = serde_json::json!(original_tool_name);
            }
        }
        
        let modified_body = serde_json::to_vec(&modified_request)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {}", e)))?;
        
        let auth_headers = build_auth_headers(state, server).await
            .map_err(|e| {
                tracing::error!("Auth error for server '{}': {}", server.name, e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::AUTH_ERROR, e))
            })?;
        
        tracing::debug!("Auth headers count: {}", auth_headers.len());
        
        return forward_request(&server.url, headers, auth_headers, modified_body).await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("{}: {}", error::PROXY_ERROR, e)));
    }
    
    tracing::warn!("Tool '{}' not found in any of {} gateway servers", tool_name, servers.len());
    Err((StatusCode::NOT_FOUND, format!("Tool '{}' not found in any gateway server", tool_name)))
}

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
    
        if !self.allowed_tools.is_empty() {
            return self.allowed_tools.iter().any(|t| t == tool_name);
        }
        
    
        if !self.denied_tools.is_empty() {
            return !self.denied_tools.iter().any(|t| t == tool_name);
        }
        
    
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
#[allow(dead_code)]
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
#[derive(Debug, Serialize, Deserialize)]
struct McpToolsListResponse {
    jsonrpc: String,
    id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<ToolsListResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
async fn get_server_by_name(state: &AppState, name: &str, user_id: Uuid) -> Result<ServerRow, String> {
    sqlx::query_as::<_, ServerRow>(SQL_SELECT_SERVER)
    .bind(name)
    .bind(user_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| format!("{}: {}", error::DATABASE_ERROR, e))?
    .ok_or_else(|| error::SERVER_NOT_FOUND.to_string())
}

async fn get_governance_config(state: &AppState, server_id: Uuid) -> Result<GovernanceConfig, String> {
    let row: Option<GovernanceRow> = sqlx::query_as(SQL_SELECT_GOVERNANCE)
    .bind(server_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| format!("{}: {}", error::DATABASE_ERROR, e))?;
    
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
fn handle_tools_call(request: &serde_json::Value, governance: &GovernanceConfig) -> Result<Vec<u8>, (StatusCode, String)> {

    let tool_name = request
        .get("params")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");
    
    if tool_name.is_empty() {
    
        return serde_json::to_vec(request)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {}", e)));
    }
    

    let original_name = governance.strip_prefix(tool_name);
    

    if !governance.is_tool_allowed(original_name) {
        return Err((
            StatusCode::FORBIDDEN,
            format!("Tool '{}' is not allowed by governance policy", original_name),
        ));
    }
    

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

async fn filter_tools_response(response: Response, governance: &GovernanceConfig) -> Result<Response, (StatusCode, String)> {

    if !governance.has_filter() && governance.tool_prefix.is_empty() {
        return Ok(response);
    }
    
    let status = response.status();
    let headers = response.headers().clone();
    

    let body_bytes = axum::body::to_bytes(response.into_body(), 10 * 1024 * 1024)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read response: {}", e)))?;
    

    let body_str = String::from_utf8_lossy(&body_bytes);
    

    if body_str.contains("event:") && body_str.contains("data:") {
    
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
    

    let mut builder = Response::builder().status(status);
    for (name, value) in headers.iter() {
        if name != "transfer-encoding" && name != "connection" {
            builder = builder.header(name.as_str(), value.to_str().unwrap_or(""));
        }
    }
    
    builder.body(Body::from(body_bytes.to_vec()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to build response: {}", e)))
}

fn filter_sse_response(body: &str, governance: &GovernanceConfig) -> String {
    let mut result = String::new();
    let mut current_event = String::new();
    let mut current_data = String::new();
    
    for line in body.lines() {
        if line.starts_with("event:") {
            current_event = line.to_string();
        } else if let Some(data) = line.strip_prefix("data:") {
            current_data = data.trim().to_string();
            
        
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
async fn build_auth_headers(state: &AppState, server: &ServerRow) -> Result<Vec<(String, String)>, String> {
    let mut headers = Vec::new();
    
    match server.auth_type.as_deref() {
        Some("none") | None => {
        
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
async fn forward_request(
    target_url: &str,
    original_headers: HeaderMap,
    auth_headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> Result<Response, String> {
    let client = reqwest::Client::new();
    let mut req_builder = client.post(target_url);
    

    if let Some(content_type) = original_headers.get("content-type") {
        req_builder = req_builder.header("Content-Type", content_type.to_str().unwrap_or("application/json"));
    } else {
        req_builder = req_builder.header("Content-Type", "application/json");
    }
    

    req_builder = req_builder.header("Accept", "application/json, text/event-stream");
    

    if let Some(session_id) = original_headers.get("mcp-session-id") {
        req_builder = req_builder.header("Mcp-Session-Id", session_id.to_str().unwrap_or(""));
    }
    

    for (name, value) in auth_headers {
        req_builder = req_builder.header(&name, &value);
    }
    
    req_builder = req_builder.body(body);
    

    let response = req_builder.send().await
        .map_err(|e| format!("Request failed: {}", e))?;
    

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    mod governance_config {
        use super::*;

        #[test]
        fn has_filter_returns_false_when_empty() {
            let config = GovernanceConfig::default();
            assert!(!config.has_filter());
        }

        #[test]
        fn has_filter_returns_true_with_allowed_tools() {
            let config = GovernanceConfig {
                allowed_tools: vec!["tool1".to_string()],
                ..Default::default()
            };
            assert!(config.has_filter());
        }

        #[test]
        fn has_filter_returns_true_with_denied_tools() {
            let config = GovernanceConfig {
                denied_tools: vec!["tool1".to_string()],
                ..Default::default()
            };
            assert!(config.has_filter());
        }

        #[test]
        fn is_tool_allowed_returns_true_when_no_filter() {
            let config = GovernanceConfig::default();
            assert!(config.is_tool_allowed("any_tool"));
        }

        #[test]
        fn is_tool_allowed_allowlist_includes_tool() {
            let config = GovernanceConfig {
                allowed_tools: vec!["allowed_tool".to_string()],
                ..Default::default()
            };
            assert!(config.is_tool_allowed("allowed_tool"));
            assert!(!config.is_tool_allowed("other_tool"));
        }

        #[test]
        fn is_tool_allowed_blocklist_excludes_tool() {
            let config = GovernanceConfig {
                denied_tools: vec!["blocked_tool".to_string()],
                ..Default::default()
            };
            assert!(!config.is_tool_allowed("blocked_tool"));
            assert!(config.is_tool_allowed("other_tool"));
        }

        #[test]
        fn add_prefix_returns_original_when_no_prefix() {
            let config = GovernanceConfig::default();
            assert_eq!(config.add_prefix("tool"), "tool");
        }

        #[test]
        fn add_prefix_prepends_prefix() {
            let config = GovernanceConfig {
                tool_prefix: "cf".to_string(),
                ..Default::default()
            };
            assert_eq!(config.add_prefix("search"), "cf_search");
        }

        #[test]
        fn strip_prefix_returns_original_when_no_prefix() {
            let config = GovernanceConfig::default();
            assert_eq!(config.strip_prefix("tool"), "tool");
        }

        #[test]
        fn strip_prefix_removes_matching_prefix() {
            let config = GovernanceConfig {
                tool_prefix: "cf".to_string(),
                ..Default::default()
            };
            assert_eq!(config.strip_prefix("cf_search"), "search");
        }

        #[test]
        fn strip_prefix_returns_original_when_prefix_not_matching() {
            let config = GovernanceConfig {
                tool_prefix: "cf".to_string(),
                ..Default::default()
            };
            assert_eq!(config.strip_prefix("other_search"), "other_search");
        }
    }

    mod handle_tools_call_tests {
        use super::*;

        #[test]
        fn allows_tool_in_allowlist() {
            let config = GovernanceConfig {
                allowed_tools: vec!["allowed_tool".to_string()],
                ..Default::default()
            };
            let request = json!({
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": { "name": "allowed_tool", "arguments": {} },
                "id": 1
            });

            let result = handle_tools_call(&request, &config);
            assert!(result.is_ok());
        }

        #[test]
        fn blocks_tool_not_in_allowlist() {
            let config = GovernanceConfig {
                allowed_tools: vec!["allowed_tool".to_string()],
                ..Default::default()
            };
            let request = json!({
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": { "name": "blocked_tool", "arguments": {} },
                "id": 1
            });

            let result = handle_tools_call(&request, &config);
            assert!(result.is_err());
        }

        #[test]
        fn strips_prefix_before_forwarding() {
            let config = GovernanceConfig {
                tool_prefix: "cf".to_string(),
                ..Default::default()
            };
            let request = json!({
                "jsonrpc": "2.0",
                "method": "tools/call",
                "params": { "name": "cf_search", "arguments": {} },
                "id": 1
            });

            let result = handle_tools_call(&request, &config).unwrap();
            let modified: serde_json::Value = serde_json::from_slice(&result).unwrap();
            assert_eq!(modified["params"]["name"], "search");
        }
    }

    mod filter_tools_tests {
        use super::*;

        fn make_tool(name: &str) -> McpTool {
            McpTool {
                name: name.to_string(),
                description: None,
                input_schema: None,
                annotations: None,
            }
        }

        #[test]
        fn returns_all_tools_when_no_filter() {
            let config = GovernanceConfig::default();
            let tools = vec![make_tool("tool1"), make_tool("tool2")];
            let result = filter_tools(&tools, &config);
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn filters_to_allowlist() {
            let config = GovernanceConfig {
                allowed_tools: vec!["tool1".to_string()],
                ..Default::default()
            };
            let tools = vec![make_tool("tool1"), make_tool("tool2")];
            let result = filter_tools(&tools, &config);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].name, "tool1");
        }

        #[test]
        fn filters_out_blocklist() {
            let config = GovernanceConfig {
                denied_tools: vec!["tool1".to_string()],
                ..Default::default()
            };
            let tools = vec![make_tool("tool1"), make_tool("tool2")];
            let result = filter_tools(&tools, &config);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].name, "tool2");
        }

        #[test]
        fn adds_prefix_to_tool_names() {
            let config = GovernanceConfig {
                tool_prefix: "cf".to_string(),
                ..Default::default()
            };
            let tools = vec![make_tool("search")];
            let result = filter_tools(&tools, &config);
            assert_eq!(result[0].name, "cf_search");
        }

        #[test]
        fn combines_filter_and_prefix() {
            let config = GovernanceConfig {
                allowed_tools: vec!["search".to_string()],
                tool_prefix: "cf".to_string(),
                ..Default::default()
            };
            let tools = vec![make_tool("search"), make_tool("migrate")];
            let result = filter_tools(&tools, &config);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].name, "cf_search");
        }
    }
}

