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
use crate::routes::auth::extract_org_context;
use crate::services::audit::{self, AuditQuery, AuditListResponse, AuditLogDetail};
use crate::messages::error;

/// GET /api/audit
/// List audit logs with pagination and filters
pub async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Json<AuditListResponse>, (StatusCode, String)> {
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let response = audit::list_audit_logs(&state.db.pool, org_id, query)
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
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let log = audit::get_audit_log(&state.db.pool, org_id, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    match log {
        Some(l) => Ok(Json(l)),
        None => Err((StatusCode::NOT_FOUND, "Audit log not found".to_string())),
    }
}
