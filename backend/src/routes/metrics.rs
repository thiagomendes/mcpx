//! Metrics API Routes
//! 
//! Exposes TimescaleDB metrics via REST API

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::AppState;
use crate::routes::auth::{extract_token, validate_token, get_dev_user_id};
use crate::services::metrics::{self, TodayMetrics, HourlyStat, TargetMetrics, MetricsQuery, QueryResponse};
use crate::messages::error;

#[derive(Debug, Deserialize)]
pub struct HourlyParams {
    #[serde(default = "default_hours")]
    pub hours: f64,
}

fn default_hours() -> f64 {
    24.0
}

async fn get_user_id(
    headers: &axum::http::HeaderMap,
    state: &AppState,
) -> Result<uuid::Uuid, (StatusCode, String)> {
    if let Some(dev_user_id) = get_dev_user_id() {
        return Ok(dev_user_id);
    }
    let token = extract_token(headers)?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;
    uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))
}

/// GET /api/metrics/today
/// Returns today's request count for servers and gateways
pub async fn get_today(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<TodayMetrics>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let metrics = metrics::get_today_count(&state.db.pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(metrics))
}

/// GET /api/metrics/hourly?hours=24
/// Returns hourly breakdown from continuous aggregate
pub async fn get_hourly(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HourlyParams>,
) -> Result<Json<Vec<HourlyStat>>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let stats = metrics::get_hourly_stats(&state.db.pool, user_id, params.hours)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(stats))
}

/// GET /api/metrics/by-target?hours=24
/// Returns metrics grouped by server/gateway
pub async fn get_by_target(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HourlyParams>,
) -> Result<Json<Vec<TargetMetrics>>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let metrics = metrics::get_metrics_by_target(&state.db.pool, user_id, params.hours)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(metrics))
}

/// POST /api/metrics/query
/// Flexible query endpoint with filters, grouping, and bucket size
pub async fn query(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(query): Json<MetricsQuery>,
) -> Result<Json<QueryResponse>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let response = metrics::query_metrics(&state.db.pool, user_id, query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct SummaryParams {
    #[serde(default = "default_hours")]
    pub hours: f64,
    pub target_name: Option<String>,
}

/// GET /api/metrics/summary?hours=6&target_name=deepwiki
/// Returns summary stats with percentiles for dashboard cards
pub async fn get_summary(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(params): Query<SummaryParams>,
) -> Result<Json<metrics::SummaryStats>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let stats = metrics::get_summary_stats(
        &state.db.pool, 
        user_id, 
        params.hours,
        params.target_name.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(stats))
}
