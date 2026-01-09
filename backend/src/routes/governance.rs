use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::messages::error;
use crate::middleware::auth::AuthUser;
use crate::AppState;

const SQL_SELECT_SERVER_ID: &str = "SELECT id FROM servers WHERE name = $1 AND org_id = $2";
const SQL_DELETE_GOVERNANCE: &str = "DELETE FROM governance_configs WHERE server_id = $1";
const SQL_SELECT_GOVERNANCE: &str = "SELECT * FROM governance_configs WHERE server_id = $1";
const SQL_UPSERT_GOVERNANCE: &str = r#"
    INSERT INTO governance_configs (server_id, allowed_tools, denied_tools, tool_prefix)
    VALUES ($1, $2, $3, $4)
    ON CONFLICT (server_id) DO UPDATE SET
        allowed_tools = EXCLUDED.allowed_tools,
        denied_tools = EXCLUDED.denied_tools,
        tool_prefix = EXCLUDED.tool_prefix,
        updated_at = NOW()
    RETURNING *
"#;

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

async fn get_server_id(
    state: &AppState,
    org_id: Uuid,
    server_name: &str,
) -> Result<Uuid, (StatusCode, String)> {
    let row: Option<(Uuid,)> = sqlx::query_as(SQL_SELECT_SERVER_ID)
        .bind(server_name)
        .bind(org_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    row.map(|(id,)| id)
        .ok_or((StatusCode::NOT_FOUND, error::SERVER_NOT_FOUND.to_string()))
}

fn json_to_vec(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

pub async fn get_governance(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(name): Path<String>,
) -> Result<Json<GovernanceResponse>, (StatusCode, String)> {
    let org_id = auth.org_id;
    let server_id = get_server_id(&state, org_id, &name).await?;

    let config: Option<GovernanceConfig> = sqlx::query_as(SQL_SELECT_GOVERNANCE)
        .bind(server_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

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

pub async fn set_governance(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(name): Path<String>,
    Json(payload): Json<CreateGovernanceRequest>,
) -> Result<(StatusCode, Json<GovernanceResponse>), (StatusCode, String)> {
    let org_id = auth.org_id;
    let server_id = get_server_id(&state, org_id, &name).await?;

    if !payload.allowed_tools.is_empty() && !payload.denied_tools.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            error::MUTUALLY_EXCLUSIVE.to_string(),
        ));
    }

    if !payload.tool_prefix.is_empty()
        && !payload
            .tool_prefix
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err((
            StatusCode::BAD_REQUEST,
            error::INVALID_PREFIX_FORMAT.to_string(),
        ));
    }

    let allowed_json = serde_json::to_value(&payload.allowed_tools).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("JSON error: {}", e),
        )
    })?;
    let denied_json = serde_json::to_value(&payload.denied_tools).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("JSON error: {}", e),
        )
    })?;

    let config: GovernanceConfig = sqlx::query_as(SQL_UPSERT_GOVERNANCE)
        .bind(server_id)
        .bind(&allowed_json)
        .bind(&denied_json)
        .bind(&payload.tool_prefix)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    // Record audit log
    let _ = crate::services::audit::record_audit_log(
        &state.db.pool,
        org_id,
        Some(auth.user_id),
        crate::services::audit::actions::GOVERNANCE_UPDATE,
        crate::services::audit::resource_types::GOVERNANCE,
        Some(config.id),
        Some(&name),
        Some(serde_json::json!({
            "allowed_tools": &payload.allowed_tools,
            "denied_tools": &payload.denied_tools,
            "tool_prefix": &payload.tool_prefix
        })),
        None,
        None,
    )
    .await;

    Ok((
        StatusCode::OK,
        Json(GovernanceResponse {
            server_name: name,
            allowed_tools: json_to_vec(&config.allowed_tools),
            denied_tools: json_to_vec(&config.denied_tools),
            tool_prefix: config.tool_prefix,
            created_at: config.created_at,
            updated_at: config.updated_at,
        }),
    ))
}

pub async fn delete_governance(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let org_id = auth.org_id;
    let server_id = get_server_id(&state, org_id, &name).await?;

    sqlx::query(SQL_DELETE_GOVERNANCE)
        .bind(server_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    // Record audit log
    let _ = crate::services::audit::record_audit_log(
        &state.db.pool,
        org_id,
        Some(auth.user_id),
        crate::services::audit::actions::GOVERNANCE_DELETE,
        crate::services::audit::resource_types::GOVERNANCE,
        Some(server_id),
        Some(&name),
        None,
        None,
        None,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
