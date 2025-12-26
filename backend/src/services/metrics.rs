//! Metrics Service using TimescaleDB
//!
//! Records and queries request metrics using TimescaleDB hypertables
//! and continuous aggregates for efficient time-series analytics.

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;

// ============================================
// TYPE ALIASES FOR COMPLEX QUERIES
// ============================================

type HourlyStatRow = (DateTime<Utc>, i64, i64, Option<i32>);
type TargetMetricsRow = (String, Uuid, String, i64, i64, Option<i32>);
type QueryMetricsRow = (Option<DateTime<Utc>>, Option<String>, Option<String>, Option<String>, i64, i64, i64, Option<i32>, Option<i32>, Option<i32>, Option<i32>, Option<i32>);
type LatencyRow = (Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, i64, Option<i64>);
type ThroughputRow = (Option<f64>, Option<f64>, Option<f64>);

/// Struct to group request metric arguments
pub struct RequestMetric<'a> {
    pub org_id: Uuid,
    pub target_type: &'a str,
    pub target_id: Uuid,
    pub target_name: &'a str,
    pub method: Option<&'a str>,
    pub tool_name: Option<&'a str>,
    pub latency_ms: i32,
    pub success: bool,
}

/// Record a request metric
pub async fn record_request(
    pool: &PgPool,
    metric: RequestMetric<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO request_metrics (time, org_id, target_type, target_id, target_name, method, tool_name, latency_ms, success)
        VALUES (NOW(), $1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(metric.org_id)
    .bind(metric.target_type)
    .bind(metric.target_id)
    .bind(metric.target_name)
    .bind(metric.method)
    .bind(metric.tool_name)
    .bind(metric.latency_ms)
    .bind(metric.success)
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
pub async fn get_today_count(pool: &PgPool, org_id: Uuid) -> Result<TodayMetrics, sqlx::Error> {
    let row: (i64, i64) = sqlx::query_as(
        r#"
        SELECT 
            COALESCE(SUM(CASE WHEN target_type = 'server' THEN 1 ELSE 0 END), 0) as servers,
            COALESCE(SUM(CASE WHEN target_type = 'gateway' THEN 1 ELSE 0 END), 0) as gateways
        FROM request_metrics
        WHERE org_id = $1 
          AND time >= time_bucket('1 day', NOW())
        "#,
    )
    .bind(org_id)
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
    org_id: Uuid,
    hours: f64,
) -> Result<Vec<HourlyStat>, sqlx::Error> {
    let rows: Vec<HourlyStatRow> = sqlx::query_as(
        r#"
        SELECT bucket, total, success_count, avg_latency_ms
        FROM request_metrics_hourly
        WHERE org_id = $1
          AND bucket >= NOW() - ($2 || ' hours')::INTERVAL
        ORDER BY bucket DESC
        "#,
    )
    .bind(org_id)
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
    org_id: Uuid,
    hours: f64,
) -> Result<Vec<TargetMetrics>, sqlx::Error> {
    let rows: Vec<TargetMetricsRow> = sqlx::query_as(
        r#"
        SELECT target_type, target_id, target_name, 
               COUNT(*)::BIGINT as total, 
               SUM(CASE WHEN success THEN 1 ELSE 0 END)::BIGINT as success_count,
               AVG(latency_ms)::INT as avg_latency_ms
        FROM request_metrics
        WHERE org_id = $1
          AND time >= NOW() - ($2 || ' hours')::INTERVAL
        GROUP BY target_type, target_id, target_name
        ORDER BY total DESC
        "#,
    )
    .bind(org_id)
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
    pub p50_latency_ms: Option<i32>,
    pub p95_latency_ms: Option<i32>,
    pub p99_latency_ms: Option<i32>,
    pub max_latency_ms: Option<i32>,
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
    org_id: Uuid,
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
        "org_id = $1".to_string(),
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
        "SELECT {}, {}, {}, {}, \
         COUNT(*)::BIGINT, \
         COUNT(*) FILTER (WHERE success)::BIGINT, \
         COUNT(*) FILTER (WHERE NOT success)::BIGINT, \
         AVG(latency_ms)::INT, \
         COALESCE(percentile_cont(0.5) WITHIN GROUP (ORDER BY latency_ms), 0)::INT, \
         COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms), 0)::INT, \
         COALESCE(percentile_cont(0.99) WITHIN GROUP (ORDER BY latency_ms), 0)::INT, \
         MAX(latency_ms)::INT \
         FROM request_metrics WHERE {} {} ORDER BY {}",
        bucket_col, target_type_col, target_name_col, tool_col,
        where_parts.join(" AND "),
        if group_parts.is_empty() { "".to_string() } else { format!("GROUP BY {}", group_parts.join(", ")) },
        if has_bucket { "1 ASC" } else { "5 DESC" }
    );
    
    tracing::info!("Executing metrics query: {}", sql);
    
    // Execute query - 12 columns (4 grouping + 8 aggregates)
    let rows: Vec<QueryMetricsRow> = 
        sqlx::query_as(&sql)
            .bind(org_id)
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
            p50_latency_ms: row.8,
            p95_latency_ms: row.9,
            p99_latency_ms: row.10,
            max_latency_ms: row.11,
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

// ============================================
// SUMMARY STATS WITH PERCENTILES
// ============================================

#[derive(Debug, Serialize)]
pub struct SummaryStats {
    // Latency stats (in ms)
    pub latency_avg: Option<f64>,
    pub latency_p50: Option<f64>,
    pub latency_p95: Option<f64>,
    pub latency_p99: Option<f64>,
    pub latency_max: Option<f64>,
    
    // Throughput stats (req/min)
    pub throughput_avg: Option<f64>,
    pub throughput_min: Option<f64>,
    pub throughput_max: Option<f64>,
    
    // Counts
    pub total_requests: i64,
    pub error_count: i64,
    pub error_rate: f64,
}

/// Get summary stats with percentiles for dashboard cards
pub async fn get_summary_stats(
    pool: &PgPool,
    org_id: Uuid,
    hours: f64,
    target_name: Option<&str>,
) -> Result<SummaryStats, sqlx::Error> {
    tracing::info!("Querying summary stats for org_id: {}", org_id);
    
    // Build WHERE clause
    let target_filter = match target_name {
        Some(name) if !name.is_empty() => format!("AND target_name = '{}'", name.replace('\'', "''")),
        _ => String::new(),
    };
    
    // Query for latency percentiles
    // COALESCE(..., 0) ensures we don't get NULLs for counts, butavgs/percentiles can be null
    let latency_sql = format!(
        r#"
        SELECT 
            AVG(latency_ms)::FLOAT8 as avg,
            PERCENTILE_CONT(0.50) WITHIN GROUP (ORDER BY latency_ms)::FLOAT8 as p50,
            PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY latency_ms)::FLOAT8 as p95,
            PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY latency_ms)::FLOAT8 as p99,
            MAX(latency_ms)::FLOAT8 as max,
            COUNT(*)::BIGINT as total,
            SUM(CASE WHEN NOT success THEN 1 ELSE 0 END)::BIGINT as errors
        FROM request_metrics
        WHERE org_id = $1
          AND time >= NOW() - ($2 || ' hours')::INTERVAL
          {}
        "#,
        target_filter
    );
    
    let latency_row: LatencyRow = 
        sqlx::query_as(&latency_sql)
            .bind(org_id)
            .bind(hours)
            .fetch_one(pool)
            .await?;
    
    // Query for throughput (requests per minute using 1-minute buckets)
    let throughput_sql = format!(
        r#"
        SELECT 
            AVG(cnt)::FLOAT8 as avg,
            MIN(cnt)::FLOAT8 as min,
            MAX(cnt)::FLOAT8 as max
        FROM (
            SELECT time_bucket('1 minute', time) as bucket, COUNT(*)::FLOAT8 as cnt
            FROM request_metrics
            WHERE org_id = $1
              AND time >= NOW() - ($2 || ' hours')::INTERVAL
              {}
            GROUP BY bucket
        ) sub
        "#,
        target_filter
    );
    
    let throughput_row: ThroughputRow = 
        sqlx::query_as(&throughput_sql)
            .bind(org_id)
            .bind(hours)
            .fetch_one(pool)
            .await?;
    
    let total_requests = latency_row.5;
    let error_count = latency_row.6.unwrap_or(0);
    let error_rate = if total_requests > 0 {
        (error_count as f64 / total_requests as f64) * 100.0
    } else {
        0.0
    };
    
    Ok(SummaryStats {
        latency_avg: latency_row.0,
        latency_p50: latency_row.1,
        latency_p95: latency_row.2,
        latency_p99: latency_row.3,
        latency_max: latency_row.4,
        throughput_avg: throughput_row.0,
        throughput_min: throughput_row.1,
        throughput_max: throughput_row.2,
        total_requests,
        error_count,
        error_rate,
    })
}
