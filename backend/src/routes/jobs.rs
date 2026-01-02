//! Background Jobs API
//!
//! Provides visibility into scheduled jobs for admins and owners.

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::auth::AuthUser;
use crate::middleware::permissions::{require_permission, Permission};
use crate::AppState;

// =============================================================================
// TYPES
// =============================================================================

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct JobSchedule {
    pub id: Uuid,
    pub job_type: String,
    pub interval_seconds: i32,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub locked_by: Option<String>,
    pub locked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub id: Uuid,
    pub job_type: String,
    pub interval_seconds: i32,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub locked_by: Option<String>,
    pub enabled: bool,
}

impl From<JobSchedule> for JobResponse {
    fn from(job: JobSchedule) -> Self {
        let status = if !job.enabled {
            "disabled".to_string()
        } else if job.locked_by.is_some() {
            "running".to_string()
        } else if job.next_run_at <= chrono::Utc::now() {
            "pending".to_string()
        } else {
            "scheduled".to_string()
        };

        Self {
            id: job.id,
            job_type: job.job_type,
            interval_seconds: job.interval_seconds,
            last_run_at: job.last_run_at,
            next_run_at: job.next_run_at,
            status,
            locked_by: job.locked_by,
            enabled: job.enabled,
        }
    }
}

// =============================================================================
// HANDLERS
// =============================================================================

/// GET /api/jobs - List all scheduled jobs (Admin/Owner only)
pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<JobResponse>>, (StatusCode, String)> {
    // Only admins and owners can view jobs
    require_permission(&auth.role, Permission::SettingsRead)?;

    let jobs: Vec<JobSchedule> = sqlx::query_as(
        "SELECT id, job_type, interval_seconds, last_run_at, next_run_at, locked_by, locked_at, enabled 
         FROM job_schedules 
         ORDER BY job_type"
    )
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(jobs.into_iter().map(JobResponse::from).collect()))
}
