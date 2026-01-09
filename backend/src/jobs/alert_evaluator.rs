#![allow(dead_code)]
//! Alert Evaluation Job
//!
//! This module is designed to be extracted to a separate service in production.
//! It's stateless and only depends on the database for state.
//!
//! To run externally:
//! 1. Set DATABASE_URL environment variable
//! 2. Call `run_evaluation_cycle` in a loop with 1-minute intervals
//!
//! The job:
//! 1. Fetches all enabled alert rules
//! 2. Queries metrics for each rule's time window
//! 3. Compares against thresholds
//! 4. Creates/resolves alert_history entries

use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Alert rule from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AlertRuleEval {
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
}

/// Result of evaluating a single rule
#[derive(Debug)]
pub struct EvalResult {
    pub rule_id: Uuid,
    pub triggered: bool,
    pub current_value: f64,
}

/// Run one evaluation cycle for all enabled rules
///
/// This function is designed to be called every minute by a scheduler.
/// It's completely stateless - all state is in the database.
pub async fn run_evaluation_cycle(pool: &PgPool) -> Result<Vec<EvalResult>, sqlx::Error> {
    let rules = get_enabled_rules(pool).await?;
    let mut results = Vec::new();

    for rule in rules {
        match evaluate_rule(pool, &rule).await {
            Ok(result) => {
                // Handle triggering/resolving
                if result.triggered {
                    let has_active = has_active_alert(pool, rule.id).await?;
                    if !has_active {
                        trigger_alert(pool, rule.id, result.current_value).await?;
                        tracing::info!(
                            "Alert triggered: {} (value: {:.2}, threshold: {})",
                            rule.name,
                            result.current_value,
                            rule.threshold
                        );
                    }
                } else {
                    // Resolve any active alerts for this rule
                    resolve_alerts_for_rule(pool, rule.id).await?;
                }
                results.push(result);
            }
            Err(e) => {
                tracing::warn!("Failed to evaluate rule {}: {}", rule.name, e);
            }
        }
    }

    Ok(results)
}

/// Get all enabled alert rules
async fn get_enabled_rules(pool: &PgPool) -> Result<Vec<AlertRuleEval>, sqlx::Error> {
    let rules = sqlx::query_as::<_, AlertRuleEval>(
        r#"
        SELECT 
            id, org_id, name, alert_type, metric, 
            scope_type, scope_id, operator, threshold, 
            duration_minutes, notify_dashboard
        FROM alert_rules 
        WHERE enabled = true
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rules)
}

/// Evaluate a single rule against current metrics
async fn evaluate_rule(pool: &PgPool, rule: &AlertRuleEval) -> Result<EvalResult, sqlx::Error> {
    let current_value = get_metric_value(pool, rule).await?;

    let triggered = match rule.operator.as_str() {
        "above" => current_value > rule.threshold,
        "below" => current_value < rule.threshold,
        _ => false,
    };

    Ok(EvalResult {
        rule_id: rule.id,
        triggered,
        current_value,
    })
}

/// Get current metric value for the rule's time window
async fn get_metric_value(pool: &PgPool, rule: &AlertRuleEval) -> Result<f64, sqlx::Error> {
    let duration_interval = format!("{} minutes", rule.duration_minutes);

    // Build scope filter
    let scope_filter = match rule.scope_type.as_str() {
        "server" | "gateway" => {
            if let Some(id) = rule.scope_id {
                format!("AND target_id = '{}'", id)
            } else {
                String::new()
            }
        }
        _ => String::new(), // "all" - no filter
    };

    let query = match rule.metric.as_str() {
        "error_rate" => format!(
            r#"
            SELECT 
                CASE WHEN COUNT(*) = 0 THEN 0.0
                ELSE (COUNT(*) FILTER (WHERE success = false)::float / COUNT(*)::float) * 100.0
                END as value
            FROM request_metrics 
            WHERE time > NOW() - INTERVAL '{}'
            {}
            "#,
            duration_interval, scope_filter
        ),
        "avg_latency" => format!(
            r#"
            SELECT COALESCE(AVG(latency_ms)::float8, 0.0) as value
            FROM request_metrics 
            WHERE time > NOW() - INTERVAL '{}'
            {}
            "#,
            duration_interval, scope_filter
        ),
        "p95_latency" => format!(
            r#"
            SELECT COALESCE(
                percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms)::float8,
                0.0
            ) as value
            FROM request_metrics 
            WHERE time > NOW() - INTERVAL '{}'
            {}
            "#,
            duration_interval, scope_filter
        ),
        "request_count" => format!(
            r#"
            SELECT COUNT(*)::float as value
            FROM request_metrics 
            WHERE time > NOW() - INTERVAL '{}'
            {}
            "#,
            duration_interval, scope_filter
        ),
        _ => return Ok(0.0),
    };

    let row = sqlx::query(&query).fetch_one(pool).await?;
    let value: f64 = row.try_get("value").unwrap_or(0.0);
    Ok(value)
}

/// Check if rule has an active (triggered or acknowledged) alert
async fn has_active_alert(pool: &PgPool, rule_id: Uuid) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM alert_history 
        WHERE rule_id = $1 AND status IN ('triggered', 'acknowledged')
        "#,
    )
    .bind(rule_id)
    .fetch_one(pool)
    .await?;

    let count: i64 = row.try_get("count").unwrap_or(0);
    Ok(count > 0)
}

/// Create a new triggered alert
async fn trigger_alert(
    pool: &PgPool,
    rule_id: Uuid,
    trigger_value: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO alert_history (rule_id, triggered_at, trigger_value, status)
        VALUES ($1, NOW(), $2, 'triggered')
        "#,
    )
    .bind(rule_id)
    .bind(trigger_value)
    .execute(pool)
    .await?;

    Ok(())
}

/// Resolve all active alerts for a rule
async fn resolve_alerts_for_rule(pool: &PgPool, rule_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE alert_history 
        SET status = 'resolved', resolved_at = NOW()
        WHERE rule_id = $1 AND status IN ('triggered', 'acknowledged')
        "#,
    )
    .bind(rule_id)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_eval_result_triggered() {
        let result = EvalResult {
            rule_id: Uuid::new_v4(),
            triggered: true,
            current_value: 10.5,
        };
        assert!(result.triggered);
    }
}
