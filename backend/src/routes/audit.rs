//! Audit API Routes
//!
//! Exposes audit logs via REST API

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::routes::auth::{extract_token, validate_token, get_dev_user_id};
use crate::services::audit::{self, AuditQuery, AuditListResponse, AuditLogDetail};
use crate::messages::error;

async fn get_user_id(
    headers: &axum::http::HeaderMap,
    state: &AppState,
) -> Result<Uuid, (StatusCode, String)> {
    if let Some(dev_user_id) = get_dev_user_id() {
        return Ok(dev_user_id);
    }
    let token = extract_token(headers)?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;
    Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))
}

/// GET /api/audit
/// List audit logs with pagination and filters
pub async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Json<AuditListResponse>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let response = audit::list_audit_logs(&state.db.pool, user_id, query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(response))
}

/// GET /api/audit/:id
/// Get audit log detail by ID
pub async fn get_audit_log(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<AuditLogDetail>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let log = audit::get_audit_log(&state.db.pool, user_id, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    match log {
        Some(l) => Ok(Json(l)),
        None => Err((StatusCode::NOT_FOUND, "Audit log not found".to_string())),
    }
}
