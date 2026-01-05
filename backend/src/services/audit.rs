//! Audit Service
//!
//! Records and queries admin/user action audit trail
//! Tracks WHO did WHAT, WHEN for security and compliance

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Record an audit log entry for admin actions
pub async fn record_audit_log(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Option<Uuid>,
    action: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
    resource_name: Option<&str>,
    details: Option<serde_json::Value>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO admin_audit_logs (
            id, org_id, user_id, action, resource_type, 
            resource_id, resource_name, details, ip_address, user_agent
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(user_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(resource_name)
    .bind(details)
    .bind(ip_address)
    .bind(user_agent)
    .execute(pool)
    .await?;

    Ok(id)
}

#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub time: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub resource_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogDetail {
    pub id: Uuid,
    pub time: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub resource_name: Option<String>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    #[serde(default = "default_limit")]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub user_id: Option<Uuid>,
    pub hours: Option<f64>,
}

fn default_limit() -> i32 {
    50
}

#[derive(Debug, Serialize)]
pub struct AuditLogListResponse {
    pub data: Vec<AuditLogEntry>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

// Type aliases for complex SQL queries
type AuditListDbRow = (
    Uuid,
    DateTime<Utc>,
    Option<Uuid>,
    String,
    String,
    Option<Uuid>,
    Option<String>,
);

type AuditDetailDbRow = (
    Uuid,
    DateTime<Utc>,
    Option<Uuid>,
    String,
    String,
    Option<Uuid>,
    Option<String>,
    Option<serde_json::Value>,
    Option<String>,
    Option<String>,
);

/// List audit logs with pagination and filters
pub async fn list_audit_logs(
    pool: &PgPool,
    org_id: Uuid,
    query: AuditLogQuery,
) -> Result<AuditLogListResponse, sqlx::Error> {
    let hours = query.hours.unwrap_or(168.0); // Default 7 days
    let limit = query.limit.min(100);
    let offset = query.offset;

    // Build WHERE clause
    let mut conditions = vec![
        "org_id = $1".to_string(),
        format!("time >= NOW() - INTERVAL '{} hours'", hours),
    ];

    if let Some(ref action) = query.action {
        conditions.push(format!("action = '{}'", action.replace('\'', "''")));
    }
    if let Some(ref resource_type) = query.resource_type {
        conditions.push(format!(
            "resource_type = '{}'",
            resource_type.replace('\'', "''")
        ));
    }
    if let Some(user_id) = query.user_id {
        conditions.push(format!("user_id = '{}'", user_id));
    }

    let where_clause = conditions.join(" AND ");

    // Get total count
    let count_sql = format!(
        "SELECT COUNT(*)::BIGINT FROM admin_audit_logs WHERE {}",
        where_clause
    );
    let total: (i64,) = sqlx::query_as(&count_sql)
        .bind(org_id)
        .fetch_one(pool)
        .await?;

    // Get paginated data
    let data_sql = format!(
        r#"
        SELECT id, time, user_id, action, resource_type, resource_id, resource_name
        FROM admin_audit_logs
        WHERE {}
        ORDER BY time DESC
        LIMIT {} OFFSET {}
        "#,
        where_clause, limit, offset
    );

    let rows: Vec<AuditListDbRow> = sqlx::query_as(&data_sql)
        .bind(org_id)
        .fetch_all(pool)
        .await?;

    let data: Vec<AuditLogEntry> = rows
        .into_iter()
        .map(|row| AuditLogEntry {
            id: row.0,
            time: row.1,
            user_id: row.2,
            action: row.3,
            resource_type: row.4,
            resource_id: row.5,
            resource_name: row.6,
        })
        .collect();

    Ok(AuditLogListResponse {
        data,
        total: total.0,
        limit,
        offset,
    })
}

/// Get audit log detail by ID
pub async fn get_audit_log(
    pool: &PgPool,
    org_id: Uuid,
    log_id: Uuid,
) -> Result<Option<AuditLogDetail>, sqlx::Error> {
    let row: Option<AuditDetailDbRow> = sqlx::query_as(
        r#"
        SELECT id, time, user_id, action, resource_type, 
               resource_id, resource_name, details, ip_address, user_agent
        FROM admin_audit_logs
        WHERE id = $1 AND org_id = $2
        "#,
    )
    .bind(log_id)
    .bind(org_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| AuditLogDetail {
        id: r.0,
        time: r.1,
        user_id: r.2,
        action: r.3,
        resource_type: r.4,
        resource_id: r.5,
        resource_name: r.6,
        details: r.7,
        ip_address: r.8,
        user_agent: r.9,
    }))
}

// ============================================================================
// Action Constants
// ============================================================================

pub mod actions {
    // Server actions
    pub const SERVER_CREATE: &str = "server.create";
    pub const SERVER_UPDATE: &str = "server.update";
    pub const SERVER_DELETE: &str = "server.delete";

    // Gateway actions
    pub const GATEWAY_CREATE: &str = "gateway.create";
    pub const GATEWAY_UPDATE: &str = "gateway.update";
    pub const GATEWAY_DELETE: &str = "gateway.delete";

    // User actions
    pub const USER_LOGIN: &str = "user.login";
    pub const USER_LOGOUT: &str = "user.logout";

    // Org actions
    pub const ORG_MEMBER_ADD: &str = "org.member.add";
    pub const ORG_MEMBER_REMOVE: &str = "org.member.remove";
}

// ============================================================================
// Resource Types
// ============================================================================

pub mod resource_types {
    pub const SERVER: &str = "server";
    pub const GATEWAY: &str = "gateway";
    pub const USER: &str = "user";
    pub const ORG: &str = "org";
}
