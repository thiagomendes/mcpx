use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::routes::auth::{extract_token, validate_token};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct GovernanceConfig {
    pub id: Uuid,
    pub server_id: Uuid,
    pub allowed_tools: serde_json::Value,
    pub denied_tools: serde_json::Value,
    pub tool_prefix: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct GovernanceResponse {
    pub server_name: String,
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
    pub tool_prefix: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGovernanceRequest {
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub denied_tools: Vec<String>,
    #[serde(default)]
    pub tool_prefix: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn get_user_id(
    headers: &axum::http::HeaderMap,
    state: &AppState,
) -> Result<Uuid, (StatusCode, String)> {
    let token = extract_token(headers)?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;
    Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))
}

async fn get_server_id(
    state: &AppState,
    user_id: Uuid,
    server_name: &str,
) -> Result<Uuid, (StatusCode, String)> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM servers WHERE user_id = $1 AND name = $2"
    )
    .bind(user_id)
    .bind(server_name)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    row.map(|(id,)| id)
        .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))
}

fn json_to_vec(value: &serde_json::Value) -> Vec<String> {
    value.as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/servers/:name/governance
pub async fn get_governance(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
) -> Result<Json<GovernanceResponse>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;
    let server_id = get_server_id(&state, user_id, &name).await?;
    
    let config: Option<GovernanceConfig> = sqlx::query_as(
        "SELECT * FROM governance_configs WHERE server_id = $1"
    )
    .bind(server_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    match config {
        Some(c) => Ok(Json(GovernanceResponse {
            server_name: name,
            allowed_tools: json_to_vec(&c.allowed_tools),
            denied_tools: json_to_vec(&c.denied_tools),
            tool_prefix: c.tool_prefix,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })),
        None => Ok(Json(GovernanceResponse {
            server_name: name,
            allowed_tools: vec![],
            denied_tools: vec![],
            tool_prefix: String::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })),
    }
}

/// POST /api/servers/:name/governance
pub async fn set_governance(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
    Json(payload): Json<CreateGovernanceRequest>,
) -> Result<(StatusCode, Json<GovernanceResponse>), (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;
    let server_id = get_server_id(&state, user_id, &name).await?;
    
    // Validate: cannot have both whitelist and blacklist
    if !payload.allowed_tools.is_empty() && !payload.denied_tools.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot specify both allowed_tools and denied_tools (use one or the other)".to_string(),
        ));
    }
    
    // Validate: tool_prefix must be alphanumeric + underscore
    if !payload.tool_prefix.is_empty() {
        if !payload.tool_prefix.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid tool_prefix: must be alphanumeric with underscores (no spaces)".to_string(),
            ));
        }
    }
    
    let allowed_json = serde_json::to_value(&payload.allowed_tools)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {}", e)))?;
    let denied_json = serde_json::to_value(&payload.denied_tools)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JSON error: {}", e)))?;
    
    // Upsert governance config
    let config: GovernanceConfig = sqlx::query_as(
        r#"
        INSERT INTO governance_configs (server_id, allowed_tools, denied_tools, tool_prefix)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (server_id) DO UPDATE SET
            allowed_tools = EXCLUDED.allowed_tools,
            denied_tools = EXCLUDED.denied_tools,
            tool_prefix = EXCLUDED.tool_prefix,
            updated_at = NOW()
        RETURNING *
        "#
    )
    .bind(server_id)
    .bind(&allowed_json)
    .bind(&denied_json)
    .bind(&payload.tool_prefix)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    Ok((StatusCode::OK, Json(GovernanceResponse {
        server_name: name,
        allowed_tools: json_to_vec(&config.allowed_tools),
        denied_tools: json_to_vec(&config.denied_tools),
        tool_prefix: config.tool_prefix,
        created_at: config.created_at,
        updated_at: config.updated_at,
    })))
}

/// DELETE /api/servers/:name/governance
pub async fn delete_governance(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;
    let server_id = get_server_id(&state, user_id, &name).await?;
    
    sqlx::query("DELETE FROM governance_configs WHERE server_id = $1")
        .bind(server_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
    
    Ok(StatusCode::NO_CONTENT)
}
