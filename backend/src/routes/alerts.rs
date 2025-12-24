//! Alerts API Routes
//!
//! CRUD endpoints for alert rules and history

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::routes::auth::{extract_token, validate_token, get_dev_user_id};
use crate::services::alerts::{
    self, AlertRule, ActiveAlert, CreateAlertRule, UpdateAlertRule,
    AlertHistoryQuery, AlertHistoryResponse,
};
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

/// GET /api/alerts - List user's alert rules
pub async fn list_rules(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<AlertRule>>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let rules = alerts::list_rules(&state.db.pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(rules))
}

/// POST /api/alerts - Create new alert rule
pub async fn create_rule(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(input): Json<CreateAlertRule>,
) -> Result<(StatusCode, Json<AlertRule>), (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    // Validate input
    if !["threshold", "spike", "no_data"].contains(&input.alert_type.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid alert_type".to_string()));
    }
    if !["error_rate", "avg_latency", "request_count", "p95_latency"].contains(&input.metric.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid metric".to_string()));
    }
    if !["all", "server", "gateway"].contains(&input.scope_type.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid scope_type".to_string()));
    }
    if !["above", "below"].contains(&input.operator.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid operator".to_string()));
    }

    let rule = alerts::create_rule(&state.db.pool, user_id, input)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok((StatusCode::CREATED, Json(rule)))
}

/// GET /api/alerts/:id - Get alert rule by ID
pub async fn get_rule(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<AlertRule>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let rule = alerts::get_rule(&state.db.pool, user_id, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    match rule {
        Some(r) => Ok(Json(r)),
        None => Err((StatusCode::NOT_FOUND, "Alert rule not found".to_string())),
    }
}

/// PUT /api/alerts/:id - Update alert rule
pub async fn update_rule(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAlertRule>,
) -> Result<Json<AlertRule>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    // Validate input if provided
    if let Some(ref alert_type) = input.alert_type {
        if !["threshold", "spike", "no_data"].contains(&alert_type.as_str()) {
            return Err((StatusCode::BAD_REQUEST, "Invalid alert_type".to_string()));
        }
    }
    if let Some(ref metric) = input.metric {
        if !["error_rate", "avg_latency", "request_count", "p95_latency"].contains(&metric.as_str()) {
            return Err((StatusCode::BAD_REQUEST, "Invalid metric".to_string()));
        }
    }

    let rule = alerts::update_rule(&state.db.pool, user_id, id, input)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    match rule {
        Some(r) => Ok(Json(r)),
        None => Err((StatusCode::NOT_FOUND, "Alert rule not found".to_string())),
    }
}

/// DELETE /api/alerts/:id - Delete alert rule
pub async fn delete_rule(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let deleted = alerts::delete_rule(&state.db.pool, user_id, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Alert rule not found".to_string()))
    }
}

/// GET /api/alerts/active - Get active (triggered) alerts
pub async fn get_active_alerts(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<ActiveAlert>>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let alerts = alerts::get_active_alerts(&state.db.pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(alerts))
}

/// GET /api/alerts/history - Get alert history with pagination
pub async fn get_history(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(query): Query<AlertHistoryQuery>,
) -> Result<Json<AlertHistoryResponse>, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let history = alerts::list_history(&state.db.pool, user_id, query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    Ok(Json(history))
}

/// POST /api/alerts/:id/acknowledge - Acknowledge an active alert
pub async fn acknowledge_alert(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = get_user_id(&headers, &state).await?;

    let acknowledged = alerts::acknowledge_alert(&state.db.pool, user_id, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    if acknowledged {
        Ok(StatusCode::OK)
    } else {
        Err((StatusCode::NOT_FOUND, "Alert not found or already acknowledged".to_string()))
    }
}
