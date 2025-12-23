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
    tool_name: Option<&str>,
    latency_ms: i32,
    success: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO request_metrics (time, user_id, target_type, target_id, target_name, method, tool_name, latency_ms, success)
        VALUES (NOW(), $1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(user_id)
    .bind(target_type)
    .bind(target_id)
    .bind(target_name)
    .bind(method)
    .bind(tool_name)
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
    hours: f64,
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

pub async fn get_metrics_by_target(
    pool: &PgPool,
    user_id: Uuid,
    hours: f64,
) -> Result<Vec<TargetMetrics>, sqlx::Error> {
    let rows: Vec<(String, Uuid, String, i64, i64, Option<i32>)> = sqlx::query_as(
        r#"
        SELECT target_type, target_id, target_name, 
               COUNT(*)::BIGINT as total, 
               SUM(CASE WHEN success THEN 1 ELSE 0 END)::BIGINT as success_count,
               AVG(latency_ms)::INT as avg_latency_ms
        FROM request_metrics
        WHERE user_id = $1
          AND recorded_at >= NOW() - ($2 || ' hours')::INTERVAL
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

// ============================================
// FLEXIBLE QUERY API
// ============================================

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterField {
    TargetType,    // "server" or "gateway"
    TargetName,    // Server/Gateway name
    Tool,          // MCP tool name (stored in method column)
    Success,       // true/false
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Eq,
    Neq,
    In,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryFilter {
    pub field: FilterField,
    pub op: FilterOp,
    pub value: Option<String>,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupByField {
    TargetType,
    TargetName,
    Tool,       // Groups by method (e.g., tools/call, prompts/list)
    ToolName,   // Groups by actual tool_name (only for tools/call)
    TimeBucket,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsQuery {
    pub time_range_hours: Option<f64>,   // e.g., 24.0 for 24h, 0.25 for 15m
    pub filters: Option<Vec<QueryFilter>>,
    pub group_by: Option<Vec<GroupByField>>,
    pub bucket_size: Option<String>,      // "5m", "15m", "1h", "1d"
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub bucket: Option<DateTime<Utc>>,
    pub target_type: Option<String>,
    pub target_name: Option<String>,
    pub tool: Option<String>,
    pub count: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub avg_latency_ms: Option<i32>,
    pub success_rate: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub data: Vec<QueryResult>,
    pub meta: QueryMeta,
}

#[derive(Debug, Serialize)]
pub struct QueryMeta {
    pub total_count: i64,
    pub time_range_hours: f64,
    pub bucket_size: String,
}

pub async fn query_metrics(
    pool: &PgPool,
    user_id: Uuid,
    query: MetricsQuery,
) -> Result<QueryResponse, sqlx::Error> {
    let hours = query.time_range_hours.unwrap_or(24.0);
    let bucket_size = query.bucket_size.clone().unwrap_or_else(|| "1h".to_string());
    
    // Convert bucket size format: "5m" -> "5 minutes", "1h" -> "1 hour", "1d" -> "1 day"
    let bucket_interval = parse_bucket_size(&bucket_size);
    
    // Determine what to group by
    let group_by = query.group_by.clone().unwrap_or_else(|| vec![GroupByField::TimeBucket]);
    
    let has_bucket = group_by.iter().any(|g| matches!(g, GroupByField::TimeBucket));
    let has_target_type = group_by.iter().any(|g| matches!(g, GroupByField::TargetType));
    let has_target_name = group_by.iter().any(|g| matches!(g, GroupByField::TargetName));
    let has_tool = group_by.iter().any(|g| matches!(g, GroupByField::Tool));
    let has_tool_name = group_by.iter().any(|g| matches!(g, GroupByField::ToolName));
    
    // Build SELECT clause - ALWAYS 8 columns in fixed order
    let bucket_col = if has_bucket { format!("time_bucket('{}', time)", bucket_interval) } else { "NULL::TIMESTAMPTZ".to_string() };
    let target_type_col = if has_target_type { "target_type".to_string() } else { "NULL::TEXT".to_string() };
    let target_name_col = if has_target_name { "target_name".to_string() } else { "NULL::TEXT".to_string() };
    let tool_col = if has_tool { "method".to_string() } else if has_tool_name { "tool_name".to_string() } else { "NULL::TEXT".to_string() };
    
    // Build GROUP BY clause
    let mut group_parts = vec![];
    if has_bucket { group_parts.push("1".to_string()); }  // reference by position
    if has_target_type { group_parts.push("2".to_string()); }
    if has_target_name { group_parts.push("3".to_string()); }
    if has_tool || has_tool_name { group_parts.push("4".to_string()); }
    
    // Build WHERE clause
    let mut where_parts = vec![
        "user_id = $1".to_string(),
        format!("time >= NOW() - INTERVAL '{} hours'", hours),
    ];
    
    if let Some(filters) = &query.filters {
        for filter in filters {
            let col = match filter.field {
                FilterField::TargetType => "target_type",
                FilterField::TargetName => "target_name",
                FilterField::Tool => "method",
                FilterField::Success => "success",
            };
            
            match (&filter.op, &filter.value, &filter.values) {
                (FilterOp::Eq, Some(v), _) => {
                    if col == "success" {
                        where_parts.push(format!("{} = {}", col, v.parse::<bool>().unwrap_or(true)));
                    } else {
                        where_parts.push(format!("{} = '{}'", col, v.replace('\'', "''")));
                    }
                }
                (FilterOp::Neq, Some(v), _) => {
                    where_parts.push(format!("{} != '{}'", col, v.replace('\'', "''")));
                }
                (FilterOp::In, _, Some(vals)) => {
                    let escaped: Vec<String> = vals.iter().map(|v| format!("'{}'", v.replace('\'', "''"))).collect();
                    where_parts.push(format!("{} IN ({})", col, escaped.join(", ")));
                }
                _ => {}
            }
        }
    }
    
    let sql = format!(
        "SELECT {}, {}, {}, {}, COUNT(*)::BIGINT, COUNT(*) FILTER (WHERE success)::BIGINT, COUNT(*) FILTER (WHERE NOT success)::BIGINT, AVG(latency_ms)::INT FROM request_metrics WHERE {} {} ORDER BY {}",
        bucket_col, target_type_col, target_name_col, tool_col,
        where_parts.join(" AND "),
        if group_parts.is_empty() { "".to_string() } else { format!("GROUP BY {}", group_parts.join(", ")) },
        if has_bucket { "1 ASC" } else { "5 DESC" }
    );
    
    tracing::info!("Executing metrics query: {}", sql);
    
    // Execute query - fixed 8 columns
    let rows: Vec<(Option<DateTime<Utc>>, Option<String>, Option<String>, Option<String>, i64, i64, i64, Option<i32>)> = 
        sqlx::query_as(&sql)
            .bind(user_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    
    let total_count: i64 = rows.iter().map(|r| r.4).sum();
    
    let data: Vec<QueryResult> = rows.into_iter().map(|row| {
        let count = row.4;
        let success_count = row.5;
        QueryResult {
            bucket: row.0,
            target_type: row.1,
            target_name: row.2,
            tool: row.3,
            count,
            success_count,
            error_count: row.6,
            avg_latency_ms: row.7,
            success_rate: if count > 0 { Some((success_count as f64 / count as f64) * 100.0) } else { None },
        }
    }).collect();
    
    Ok(QueryResponse {
        data,
        meta: QueryMeta {
            total_count,
            time_range_hours: hours,
            bucket_size,
        },
    })
}

fn parse_bucket_size(s: &str) -> String {
    if s.ends_with('m') {
        format!("{} minutes", s.trim_end_matches('m'))
    } else if s.ends_with('h') {
        format!("{} hours", s.trim_end_matches('h'))
    } else if s.ends_with('d') {
        format!("{} days", s.trim_end_matches('d'))
    } else {
        s.to_string()
    }
}

