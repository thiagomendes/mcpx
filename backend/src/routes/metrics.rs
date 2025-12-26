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
use crate::routes::auth::extract_org_context;
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

/// GET /api/metrics/today
/// Returns today's request count for servers and gateways
pub async fn get_today(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<TodayMetrics>, (StatusCode, String)> {
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let metrics = metrics::get_today_count(&state.db.pool, org_id)
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
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let stats = metrics::get_hourly_stats(&state.db.pool, org_id, params.hours)
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
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let metrics = metrics::get_metrics_by_target(&state.db.pool, org_id, params.hours)
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
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let response = metrics::query_metrics(&state.db.pool, org_id, query)
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
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let stats = metrics::get_summary_stats(
        &state.db.pool, 
        org_id, 
        params.hours,
        params.target_name.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(stats))
}
