//! Metrics Service using TimescaleDB
//!
//! Records and queries request metrics using TimescaleDB hypertables
//! and continuous aggregates for efficient time-series analytics.

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Record a request metric
pub async fn record_request(
    pool: &PgPool,
    user_id: Uuid,
    target_type: &str,
    target_id: Uuid,
    target_name: &str,
    method: Option<&str>,
    latency_ms: i32,
    success: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO request_metrics (time, user_id, target_type, target_id, target_name, method, latency_ms, success)
        VALUES (NOW(), $1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(user_id)
    .bind(target_type)
    .bind(target_id)
    .bind(target_name)
    .bind(method)
    .bind(latency_ms)
    .bind(success)
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct TodayMetrics {
    pub servers: i64,
    pub gateways: i64,
}

/// Get today's request count by target type using TimescaleDB time_bucket
pub async fn get_today_count(pool: &PgPool, user_id: Uuid) -> Result<TodayMetrics, sqlx::Error> {
    let row: (i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COALESCE(SUM(CASE WHEN target_type = 'server' THEN 1 ELSE 0 END), 0) as servers,
            COALESCE(SUM(CASE WHEN target_type = 'gateway' THEN 1 ELSE 0 END), 0) as gateways
        FROM request_metrics
        WHERE user_id = $1 
          AND time >= time_bucket('1 day', NOW())
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(TodayMetrics {
        servers: row.0,
        gateways: row.1,
    })
}

#[derive(Debug, Serialize)]
pub struct HourlyStat {
    pub bucket: DateTime<Utc>,
    pub total: i64,
    pub success_count: i64,
    pub avg_latency_ms: Option<i32>,
}

/// Get hourly stats using continuous aggregate
pub async fn get_hourly_stats(
    pool: &PgPool,
    user_id: Uuid,
    hours: i32,
) -> Result<Vec<HourlyStat>, sqlx::Error> {
    let rows: Vec<(DateTime<Utc>, i64, i64, Option<i32>)> = sqlx::query_as(
        r#"
        SELECT bucket, total, success_count, avg_latency_ms
        FROM request_metrics_hourly
        WHERE user_id = $1
          AND bucket >= NOW() - ($2 || ' hours')::INTERVAL
        ORDER BY bucket DESC
        "#,
    )
    .bind(user_id)
    .bind(hours)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(bucket, total, success_count, avg_latency_ms)| HourlyStat {
            bucket,
            total,
            success_count,
            avg_latency_ms,
        })
        .collect())
}

#[derive(Debug, Serialize)]
pub struct TargetMetrics {
    pub target_type: String,
    pub target_id: Uuid,
    pub target_name: String,
    pub total: i64,
    pub success_count: i64,
    pub avg_latency_ms: Option<i32>,
}

/// Get metrics grouped by target (server/gateway)
pub async fn get_metrics_by_target(
    pool: &PgPool,
    user_id: Uuid,
    hours: i32,
) -> Result<Vec<TargetMetrics>, sqlx::Error> {
    let rows: Vec<(String, Uuid, String, i64, i64, Option<i32>)> = sqlx::query_as(
        r#"
        SELECT target_type, target_id, target_name, 
               SUM(total)::BIGINT as total, 
               SUM(success_count)::BIGINT as success_count,
               AVG(avg_latency_ms)::INT as avg_latency_ms
        FROM request_metrics_hourly
        WHERE user_id = $1
          AND bucket >= NOW() - ($2 || ' hours')::INTERVAL
        GROUP BY target_type, target_id, target_name
        ORDER BY total DESC
        "#,
    )
    .bind(user_id)
    .bind(hours)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(target_type, target_id, target_name, total, success_count, avg_latency_ms)| {
            TargetMetrics {
                target_type,
                target_id,
                target_name,
                total,
                success_count,
                avg_latency_ms,
            }
        })
        .collect())
}
