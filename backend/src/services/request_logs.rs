//! Request Logs Service
//!
//! Records and queries detailed MCP request/response logs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Record a request log entry
#[allow(clippy::too_many_arguments)]
pub async fn record_request_log(
    pool: &PgPool,
    org_id: Uuid,
    target_type: &str,
    target_id: Uuid,
    target_name: &str,
    method: Option<&str>,
    tool_name: Option<&str>,
    request_body: Option<serde_json::Value>,
    response_body: Option<serde_json::Value>,
    status_code: i32,
    error_message: Option<&str>,
    latency_ms: i32,
    success: bool,
    session_id: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            id, time, org_id, target_type, target_id, target_name,
            method, tool_name, request_body, response_body,
            status_code, error_message, latency_ms, success, session_id
        )
        VALUES ($1, NOW(), $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(target_type)
    .bind(target_id)
    .bind(target_name)
    .bind(method)
    .bind(tool_name)
    .bind(request_body)
    .bind(response_body)
    .bind(status_code)
    .bind(error_message)
    .bind(latency_ms)
    .bind(success)
    .bind(session_id)
    .execute(pool)
    .await?;

    Ok(id)
}

#[derive(Debug, Serialize)]
pub struct RequestLogEntry {
    pub id: Uuid,
    pub time: DateTime<Utc>,
    pub target_type: String,
    pub target_id: Uuid,
    pub target_name: String,
    pub method: Option<String>,
    pub tool_name: Option<String>,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i32>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RequestLogDetail {
    pub id: Uuid,
    pub time: DateTime<Utc>,
    pub target_type: String,
    pub target_id: Uuid,
    pub target_name: String,
    pub method: Option<String>,
    pub tool_name: Option<String>,
    pub request_body: Option<serde_json::Value>,
    pub response_body: Option<serde_json::Value>,
    pub status_code: Option<i32>,
    pub error_message: Option<String>,
    pub latency_ms: Option<i32>,
    pub success: bool,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RequestLogQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
    pub target_name: Option<String>,
    pub method: Option<String>,
    pub tool_name: Option<String>,
    pub success: Option<bool>,
    pub hours: Option<f64>,
}

fn default_limit() -> i32 {
    50
}

#[derive(Debug, Serialize)]
pub struct RequestLogListResponse {
    pub data: Vec<RequestLogEntry>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

// Type aliases for complex SQL queries
type RequestListDbRow = (
    Uuid,
    DateTime<Utc>,
    String,
    Uuid,
    String,
    Option<String>,
    Option<String>,
    Option<i32>,
    Option<i32>,
    bool,
    Option<String>,
);
type RequestDetailDbRow = (
    Uuid,
    DateTime<Utc>,
    String,
    Uuid,
    String,
    Option<String>,
    Option<String>,
    Option<serde_json::Value>,
    Option<serde_json::Value>,
    Option<i32>,
    Option<String>,
    Option<i32>,
    bool,
    Option<String>,
);

/// List request logs with pagination and filters
pub async fn list_request_logs(
    pool: &PgPool,
    org_id: Uuid,
    query: RequestLogQuery,
) -> Result<RequestLogListResponse, sqlx::Error> {
    let hours = query.hours.unwrap_or(24.0);
    let limit = query.limit.min(100); // Max 100 per page
    let offset = query.offset;

    // Build WHERE clause
    let mut conditions = vec![
        "org_id = $1".to_string(),
        format!("time >= NOW() - INTERVAL '{} hours'", hours),
    ];

    if let Some(ref target_name) = query.target_name {
        conditions.push(format!(
            "target_name = '{}'",
            target_name.replace('\'', "''")
        ));
    }
    if let Some(ref method) = query.method {
        conditions.push(format!("method = '{}'", method.replace('\'', "''")));
    }
    if let Some(ref tool_name) = query.tool_name {
        conditions.push(format!("tool_name = '{}'", tool_name.replace('\'', "''")));
    }
    if let Some(success) = query.success {
        conditions.push(format!("success = {}", success));
    }

    let where_clause = conditions.join(" AND ");

    // Get total count
    let count_sql = format!(
        "SELECT COUNT(*)::BIGINT FROM audit_logs WHERE {}",
        where_clause
    );
    let total: (i64,) = sqlx::query_as(&count_sql)
        .bind(org_id)
        .fetch_one(pool)
        .await?;

    // Get paginated data
    let data_sql = format!(
        r#"
        SELECT id, time, target_type, target_id, target_name, 
               method, tool_name, status_code, latency_ms, success, error_message
        FROM audit_logs
        WHERE {}
        ORDER BY time DESC
        LIMIT {} OFFSET {}
        "#,
        where_clause, limit, offset
    );

    let rows: Vec<RequestListDbRow> = sqlx::query_as(&data_sql)
        .bind(org_id)
        .fetch_all(pool)
        .await?;

    let data: Vec<RequestLogEntry> = rows
        .into_iter()
        .map(|row| RequestLogEntry {
            id: row.0,
            time: row.1,
            target_type: row.2,
            target_id: row.3,
            target_name: row.4,
            method: row.5,
            tool_name: row.6,
            status_code: row.7,
            latency_ms: row.8,
            success: row.9,
            error_message: row.10,
        })
        .collect();

    Ok(RequestLogListResponse {
        data,
        total: total.0,
        limit,
        offset,
    })
}

/// Get request log detail by ID
pub async fn get_request_log(
    pool: &PgPool,
    org_id: Uuid,
    log_id: Uuid,
) -> Result<Option<RequestLogDetail>, sqlx::Error> {
    let row: Option<RequestDetailDbRow> = sqlx::query_as(
        r#"
            SELECT id, time, target_type, target_id, target_name,
                   method, tool_name, request_body, response_body,
                   status_code, error_message, latency_ms, success, session_id
            FROM audit_logs
            WHERE id = $1 AND org_id = $2
            "#,
    )
    .bind(log_id)
    .bind(org_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| RequestLogDetail {
        id: r.0,
        time: r.1,
        target_type: r.2,
        target_id: r.3,
        target_name: r.4,
        method: r.5,
        tool_name: r.6,
        request_body: r.7,
        response_body: r.8,
        status_code: r.9,
        error_message: r.10,
        latency_ms: r.11,
        success: r.12,
        session_id: r.13,
    }))
}
