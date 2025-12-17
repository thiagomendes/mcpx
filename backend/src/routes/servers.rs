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

    let base_url = "http://localhost:8080"; // TODO: from config
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

    let server = sqlx::query_as::<_, Server>(
        r#"
        INSERT INTO servers (user_id, name, url, transport)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(&payload.name)
    .bind(&payload.url)
    .bind(&payload.transport)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate key") {
            (StatusCode::CONFLICT, "Server with this name already exists".to_string())
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
        }
    })?;

    let base_url = "http://localhost:8080";
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

    let base_url = "http://localhost:8080";
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

    let base_url = "http://localhost:8080";
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

// POST /api/servers/:name/test - Test server connectivity
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

    // Test MCP initialize endpoint
    let start = std::time::Instant::now();
    let client = reqwest::Client::new();
    
    let mcp_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "mcpx-test",
                "version": "1.0.0"
            }
        }
    });

    let result = client
        .post(&server.url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&mcp_request)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    let latency_ms = start.elapsed().as_millis() as u32;

    match result {
        Ok(response) => {
            let status = response.status().as_u16();
            if status >= 200 && status < 300 {
                Ok(Json(TestResult {
                    success: true,
                    message: "Connection successful".to_string(),
                    latency_ms,
                    status_code: Some(status),
                }))
            } else {
                Ok(Json(TestResult {
                    success: false,
                    message: format!("Server returned status {}", status),
                    latency_ms,
                    status_code: Some(status),
                }))
            }
        }
        Err(e) => {
            Ok(Json(TestResult {
                success: false,
                message: format!("Connection failed: {}", e),
                latency_ms,
                status_code: None,
            }))
        }
    }
}

#[derive(serde::Serialize)]
pub struct TestResult {
    success: bool,
    message: String,
    latency_ms: u32,
    status_code: Option<u16>,
}
