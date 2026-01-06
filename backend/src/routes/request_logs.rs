//! Request Logs API Routes
//!
//! Exposes MCP request/response logs via REST API

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::messages::error;
use crate::middleware::auth::AuthUser;
use crate::services::request_logs::{self, RequestLogListResponse, RequestLogDetail, RequestLogQuery};
use crate::AppState;

/// GET /api/request-logs
/// List request logs with pagination and filters
pub async fn list_request_logs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<RequestLogQuery>,
) -> Result<Json<RequestLogListResponse>, (StatusCode, String)> {
    let org_id = auth.org_id;

    let response = request_logs::list_request_logs(&state.db.pool, org_id, query)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    Ok(Json(response))
}

/// GET /api/request-logs/:id
/// Get request log detail by ID
pub async fn get_request_log(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<RequestLogDetail>, (StatusCode, String)> {
    let org_id = auth.org_id;

    let log = request_logs::get_request_log(&state.db.pool, org_id, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    match log {
        Some(l) => Ok(Json(l)),
        None => Err((StatusCode::NOT_FOUND, "Request log not found".to_string())),
    }
}
