//! Audit Logs API Routes
//!
//! Exposes admin/user action audit trail via REST API

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::messages::error;
use crate::middleware::auth::AuthUser;
use crate::services::audit::{self, AuditLogListResponse, AuditLogDetail, AuditLogQuery};
use crate::AppState;

/// GET /api/audit-logs
/// List audit logs with pagination and filters
pub async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<AuditLogListResponse>, (StatusCode, String)> {
    // Check permission - owner/admin only
    auth.require(crate::middleware::permissions::Permission::AuditLogsRead)?;

    let org_id = auth.org_id;

    let response = audit::list_audit_logs(&state.db.pool, org_id, query)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    Ok(Json(response))
}

/// GET /api/audit-logs/:id
/// Get audit log detail by ID
pub async fn get_audit_log(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AuditLogDetail>, (StatusCode, String)> {
    // Check permission - owner/admin only
    auth.require(crate::middleware::permissions::Permission::AuditLogsRead)?;

    let org_id = auth.org_id;

    let log = audit::get_audit_log(&state.db.pool, org_id, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    match log {
        Some(l) => Ok(Json(l)),
        None => Err((StatusCode::NOT_FOUND, "Audit log not found".to_string())),
    }
}
