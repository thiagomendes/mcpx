#![allow(dead_code)]
//! Alerts Service
//!
//! Manages alert rules and their evaluation

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertRule {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub alert_type: String,
    pub metric: String,
    pub scope_type: String,
    pub scope_id: Option<Uuid>,
    pub operator: String,
    pub threshold: f64,
    pub duration_minutes: i32,
    pub notify_dashboard: bool,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertHistory {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub trigger_value: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActiveAlert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub rule_name: String,
    pub alert_type: String,
    pub metric: String,
    pub scope_type: String,
    pub scope_id: Option<Uuid>,
    pub scope_name: Option<String>,
    pub triggered_at: DateTime<Utc>,
    pub trigger_value: f64,
    pub threshold: f64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAlertRule {
    pub name: String,
    pub alert_type: String,
    pub metric: String,
    pub scope_type: String,
    pub scope_id: Option<Uuid>,
    pub operator: String,
    pub threshold: f64,
    pub duration_minutes: i32,
    pub notify_dashboard: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAlertRule {
    pub name: Option<String>,
    pub alert_type: Option<String>,
    pub metric: Option<String>,
    pub scope_type: Option<String>,
    pub scope_id: Option<Uuid>,
    pub operator: Option<String>,
    pub threshold: Option<f64>,
    pub duration_minutes: Option<i32>,
    pub notify_dashboard: Option<bool>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AlertHistoryQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
    pub status: Option<String>,
}

fn default_limit() -> i32 {
    50
}

#[derive(Debug, Serialize)]
pub struct AlertHistoryResponse {
    pub data: Vec<AlertHistory>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

// ============================================================================
// CRUD Operations
// ============================================================================

/// Create a new alert rule
pub async fn create_rule(
    pool: &PgPool,
    org_id: Uuid,
    input: CreateAlertRule,
) -> Result<AlertRule, sqlx::Error> {
    let id = Uuid::new_v4();
    let notify = input.notify_dashboard.unwrap_or(true);
    
    sqlx::query_as::<_, AlertRule>(
        r#"
        INSERT INTO alert_rules (
            id, org_id, name, alert_type, metric, scope_type, scope_id,
            operator, threshold, duration_minutes, notify_dashboard
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, org_id, name, alert_type, metric, scope_type, scope_id,
                  operator, threshold, duration_minutes, notify_dashboard, enabled,
                  created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(&input.name)
    .bind(&input.alert_type)
    .bind(&input.metric)
    .bind(&input.scope_type)
    .bind(input.scope_id)
    .bind(&input.operator)
    .bind(input.threshold)
    .bind(input.duration_minutes)
    .bind(notify)
    .fetch_one(pool)
    .await
}

/// List all alert rules for a user
pub async fn list_rules(pool: &PgPool, org_id: Uuid) -> Result<Vec<AlertRule>, sqlx::Error> {
    sqlx::query_as::<_, AlertRule>(
        r#"
        SELECT id, org_id, name, alert_type, metric, scope_type, scope_id,
               operator, threshold, duration_minutes, notify_dashboard, enabled,
               created_at, updated_at
        FROM alert_rules
        WHERE org_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
}

/// Get a single alert rule
pub async fn get_rule(pool: &PgPool, org_id: Uuid, rule_id: Uuid) -> Result<Option<AlertRule>, sqlx::Error> {
    sqlx::query_as::<_, AlertRule>(
        r#"
        SELECT id, org_id, name, alert_type, metric, scope_type, scope_id,
               operator, threshold, duration_minutes, notify_dashboard, enabled,
               created_at, updated_at
        FROM alert_rules
        WHERE id = $1 AND org_id = $2
        "#,
    )
    .bind(rule_id)
    .bind(org_id)
    .fetch_optional(pool)
    .await
}

/// Update an alert rule
pub async fn update_rule(
    pool: &PgPool,
    org_id: Uuid,
    rule_id: Uuid,
    input: UpdateAlertRule,
) -> Result<Option<AlertRule>, sqlx::Error> {
    // Build dynamic UPDATE query
    let mut updates = Vec::new();
    let mut param_count = 2;
    
    if input.name.is_some() { param_count += 1; updates.push(format!("name = ${}", param_count)); }
    if input.alert_type.is_some() { param_count += 1; updates.push(format!("alert_type = ${}", param_count)); }
    if input.metric.is_some() { param_count += 1; updates.push(format!("metric = ${}", param_count)); }
    if input.scope_type.is_some() { param_count += 1; updates.push(format!("scope_type = ${}", param_count)); }
    if input.scope_id.is_some() { param_count += 1; updates.push(format!("scope_id = ${}", param_count)); }
    if input.operator.is_some() { param_count += 1; updates.push(format!("operator = ${}", param_count)); }
    if input.threshold.is_some() { param_count += 1; updates.push(format!("threshold = ${}", param_count)); }
    if input.duration_minutes.is_some() { param_count += 1; updates.push(format!("duration_minutes = ${}", param_count)); }
    if input.notify_dashboard.is_some() { param_count += 1; updates.push(format!("notify_dashboard = ${}", param_count)); }
    if input.enabled.is_some() { param_count += 1; updates.push(format!("enabled = ${}", param_count)); }
    
    if updates.is_empty() {
        return get_rule(pool, org_id, rule_id).await;
    }
    
    let sql = format!(
        r#"
        UPDATE alert_rules
        SET {}
        WHERE id = $1 AND org_id = $2
        RETURNING id, org_id, name, alert_type, metric, scope_type, scope_id,
                  operator, threshold, duration_minutes, notify_dashboard, enabled,
                  created_at, updated_at
        "#,
        updates.join(", ")
    );
    
    let mut query = sqlx::query_as::<_, AlertRule>(&sql)
        .bind(rule_id)
        .bind(org_id);
    
    if let Some(v) = &input.name { query = query.bind(v); }
    if let Some(v) = &input.alert_type { query = query.bind(v); }
    if let Some(v) = &input.metric { query = query.bind(v); }
    if let Some(v) = &input.scope_type { query = query.bind(v); }
    if let Some(v) = &input.scope_id { query = query.bind(v); }
    if let Some(v) = &input.operator { query = query.bind(v); }
    if let Some(v) = &input.threshold { query = query.bind(v); }
    if let Some(v) = &input.duration_minutes { query = query.bind(v); }
    if let Some(v) = &input.notify_dashboard { query = query.bind(v); }
    if let Some(v) = &input.enabled { query = query.bind(v); }
    
    query.fetch_optional(pool).await
}

/// Delete an alert rule
pub async fn delete_rule(pool: &PgPool, org_id: Uuid, rule_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM alert_rules WHERE id = $1 AND org_id = $2"
    )
    .bind(rule_id)
    .bind(org_id)
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected() > 0)
}

// ============================================================================
// Alert History Operations
// ============================================================================

/// Get active alerts for a user (dashboard panel)
pub async fn get_active_alerts(pool: &PgPool, org_id: Uuid) -> Result<Vec<ActiveAlert>, sqlx::Error> {
    sqlx::query_as::<_, ActiveAlert>(
        r#"
        SELECT h.id, h.rule_id, r.name as rule_name, r.alert_type, r.metric,
               r.scope_type, r.scope_id,
               COALESCE(s.name, g.name) as scope_name,
               h.triggered_at, h.trigger_value,
               r.threshold, h.status
        FROM alert_history h
        JOIN alert_rules r ON r.id = h.rule_id
        LEFT JOIN servers s ON r.scope_type = 'server' AND r.scope_id = s.id
        LEFT JOIN gateways g ON r.scope_type = 'gateway' AND r.scope_id = g.id
        WHERE r.org_id = $1 AND h.status IN ('triggered', 'acknowledged')
        ORDER BY h.triggered_at DESC
        "#,
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
}

/// Get alert history with pagination
pub async fn list_history(
    pool: &PgPool,
    org_id: Uuid,
    query: AlertHistoryQuery,
) -> Result<AlertHistoryResponse, sqlx::Error> {
    let limit = query.limit.min(100);
    let offset = query.offset;
    
    // Count total
    let count_sql = if query.status.is_some() {
        r#"
        SELECT COUNT(*)::BIGINT FROM alert_history h
        JOIN alert_rules r ON r.id = h.rule_id
        WHERE r.org_id = $1 AND h.status = $2
        "#
    } else {
        r#"
        SELECT COUNT(*)::BIGINT FROM alert_history h
        JOIN alert_rules r ON r.id = h.rule_id
        WHERE r.org_id = $1
        "#
    };
    
    let total: (i64,) = if let Some(ref status) = query.status {
        sqlx::query_as(count_sql)
            .bind(org_id)
            .bind(status)
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_as(count_sql)
            .bind(org_id)
            .fetch_one(pool)
            .await?
    };
    
    // Get data
    let data_sql = if query.status.is_some() {
        format!(
            r#"
            SELECT h.id, h.rule_id, h.triggered_at, h.resolved_at, h.trigger_value, h.status
            FROM alert_history h
            JOIN alert_rules r ON r.id = h.rule_id
            WHERE r.org_id = $1 AND h.status = $2
            ORDER BY h.triggered_at DESC
            LIMIT {} OFFSET {}
            "#,
            limit, offset
        )
    } else {
        format!(
            r#"
            SELECT h.id, h.rule_id, h.triggered_at, h.resolved_at, h.trigger_value, h.status
            FROM alert_history h
            JOIN alert_rules r ON r.id = h.rule_id
            WHERE r.org_id = $1
            ORDER BY h.triggered_at DESC
            LIMIT {} OFFSET {}
            "#,
            limit, offset
        )
    };
    
    let data: Vec<AlertHistory> = if let Some(ref status) = query.status {
        sqlx::query_as(&data_sql)
            .bind(org_id)
            .bind(status)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as(&data_sql)
            .bind(org_id)
            .fetch_all(pool)
            .await?
    };
    
    Ok(AlertHistoryResponse {
        data,
        total: total.0,
        limit,
        offset,
    })
}

/// Acknowledge an alert
pub async fn acknowledge_alert(
    pool: &PgPool,
    org_id: Uuid,
    alert_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE alert_history h
        SET status = 'acknowledged'
        FROM alert_rules r
        WHERE h.id = $1 AND h.rule_id = r.id AND r.org_id = $2 AND h.status = 'triggered'
        "#
    )
    .bind(alert_id)
    .bind(org_id)
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected() > 0)
}

// ============================================================================
// Alert Evaluation (Background Job)
// ============================================================================

/// Trigger a new alert
pub async fn trigger_alert(
    pool: &PgPool,
    rule_id: Uuid,
    trigger_value: f64,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    
    sqlx::query(
        r#"
        INSERT INTO alert_history (id, rule_id, trigger_value, status)
        VALUES ($1, $2, $3, 'triggered')
        "#,
    )
    .bind(id)
    .bind(rule_id)
    .bind(trigger_value)
    .execute(pool)
    .await?;
    
    Ok(id)
}

/// Resolve an active alert
pub async fn resolve_alert(pool: &PgPool, rule_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE alert_history
        SET status = 'resolved', resolved_at = NOW()
        WHERE rule_id = $1 AND status IN ('triggered', 'acknowledged')
        "#
    )
    .bind(rule_id)
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected() > 0)
}

/// Check if a rule has an active alert
pub async fn has_active_alert(pool: &PgPool, rule_id: Uuid) -> Result<bool, sqlx::Error> {
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::BIGINT FROM alert_history WHERE rule_id = $1 AND status IN ('triggered', 'acknowledged')"
    )
    .bind(rule_id)
    .fetch_one(pool)
    .await?;
    
    Ok(result.0 > 0)
}

/// Get all enabled rules for evaluation
pub async fn get_enabled_rules(pool: &PgPool) -> Result<Vec<AlertRule>, sqlx::Error> {
    sqlx::query_as::<_, AlertRule>(
        r#"
        SELECT id, org_id, name, alert_type, metric, scope_type, scope_id,
               operator, threshold, duration_minutes, notify_dashboard, enabled,
               created_at, updated_at
        FROM alert_rules
        WHERE enabled = true
        "#,
    )
    .fetch_all(pool)
    .await
}
