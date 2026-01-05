use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::messages::error;
use crate::middleware::auth::AuthUser;
use crate::middleware::permissions::Permission;
use crate::AppState;

// ============================================
// SQL QUERIES - Now using org_id instead of user_id
// ============================================

const SQL_SELECT_GATEWAYS: &str = r#"
    SELECT g.id, g.name, g.slug, g.enabled, g.created_at, g.updated_at,
           COALESCE(json_agg(
               json_build_object('name', s.name, 'priority', gs.priority)
               ORDER BY gs.priority
           ) FILTER (WHERE s.id IS NOT NULL), '[]') as servers
    FROM gateways g
    LEFT JOIN gateway_servers gs ON gs.gateway_id = g.id
    LEFT JOIN servers s ON s.id = gs.server_id
    WHERE g.org_id = $1
    GROUP BY g.id
    ORDER BY g.created_at DESC
"#;

const SQL_SELECT_GATEWAY_BY_SLUG: &str = r#"
    SELECT g.id, g.name, g.slug, g.enabled, g.created_at, g.updated_at,
           COALESCE(json_agg(
               json_build_object('name', s.name, 'priority', gs.priority)
               ORDER BY gs.priority
           ) FILTER (WHERE s.id IS NOT NULL), '[]') as servers
    FROM gateways g
    LEFT JOIN gateway_servers gs ON gs.gateway_id = g.id
    LEFT JOIN servers s ON s.id = gs.server_id
    WHERE g.org_id = $1 AND g.slug = $2
    GROUP BY g.id
"#;

const SQL_INSERT_GATEWAY: &str = r#"
    INSERT INTO gateways (org_id, name, slug, enabled)
    VALUES ($1, $2, $3, $4)
    RETURNING id, name, slug, enabled, created_at, updated_at
"#;

const SQL_UPDATE_GATEWAY: &str = r#"
    UPDATE gateways SET name = $1, slug = $2, enabled = $3, updated_at = NOW()
    WHERE id = $4 AND org_id = $5
    RETURNING id, name, slug, enabled, created_at, updated_at
"#;

const SQL_DELETE_GATEWAY: &str = "DELETE FROM gateways WHERE slug = $1 AND org_id = $2";

const SQL_ADD_SERVER_TO_GATEWAY: &str = r#"
    INSERT INTO gateway_servers (gateway_id, server_id, priority)
    SELECT $1, s.id, COALESCE((SELECT MAX(priority) + 1 FROM gateway_servers WHERE gateway_id = $1), 0)
    FROM servers s WHERE s.name = $2 AND s.org_id = $3
    ON CONFLICT (gateway_id, server_id) DO NOTHING
"#;

const SQL_REMOVE_SERVER_FROM_GATEWAY: &str = r#"
    DELETE FROM gateway_servers 
    WHERE gateway_id = $1 AND server_id = (SELECT id FROM servers WHERE name = $2 AND org_id = $3)
"#;

const SQL_GET_GATEWAY_ID: &str = "SELECT id FROM gateways WHERE slug = $1 AND org_id = $2";

// ============================================
// TYPES
// ============================================

type GatewayDbRow = (
    Uuid,
    String,
    String,
    bool,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
    serde_json::Value,
);

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GatewayRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct GatewayServer {
    pub name: String,
    pub priority: i32,
}

#[derive(Debug, Serialize)]
pub struct GatewayResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub enabled: bool,
    pub proxy_url: String,
    pub servers: Vec<GatewayServer>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGatewayRequest {
    pub name: String,
    pub slug: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub servers: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateGatewayRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddServerRequest {
    pub server_name: String,
}

// ============================================
// HELPERS
// ============================================

fn parse_servers_json(json: serde_json::Value) -> Vec<GatewayServer> {
    json.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    Some(GatewayServer {
                        name: v.get("name")?.as_str()?.to_string(),
                        priority: v.get("priority")?.as_i64()? as i32,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

// ============================================
// ROUTE HANDLERS
// ============================================

pub async fn list_gateways(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<GatewayResponse>>, (StatusCode, String)> {
    let org_id = auth.org_id;
    let org_slug = auth.org_slug;

    let rows: Vec<GatewayDbRow> = sqlx::query_as(SQL_SELECT_GATEWAYS)
        .bind(org_id)
        .fetch_all(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let gateways: Vec<GatewayResponse> = rows
        .into_iter()
        .map(
            |(id, name, slug, enabled, created_at, updated_at, servers_json)| GatewayResponse {
                id,
                name,
                slug: slug.clone(),
                enabled,
                proxy_url: format!("{}/mcp/{}/{}", state.config.base_url, org_slug, slug),
                servers: parse_servers_json(servers_json),
                created_at,
                updated_at,
            },
        )
        .collect();

    Ok(Json(gateways))
}

pub async fn get_gateway(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<GatewayResponse>, (StatusCode, String)> {
    let org_id = auth.org_id;
    let org_slug = auth.org_slug;

    let row: Option<GatewayDbRow> = sqlx::query_as(SQL_SELECT_GATEWAY_BY_SLUG)
        .bind(org_id)
        .bind(&slug)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    match row {
        Some((id, name, slug, enabled, created_at, updated_at, servers_json)) => {
            Ok(Json(GatewayResponse {
                id,
                name,
                slug: slug.clone(),
                enabled,
                proxy_url: format!("{}/mcp/{}/{}", state.config.base_url, org_slug, slug),
                servers: parse_servers_json(servers_json),
                created_at,
                updated_at,
            }))
        }
        None => Err((StatusCode::NOT_FOUND, "Gateway not found".to_string())),
    }
}

pub async fn create_gateway(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateGatewayRequest>,
) -> Result<(StatusCode, Json<GatewayResponse>), (StatusCode, String)> {
    // Check permission
    auth.require(Permission::GatewaysWrite)?;

    let org_id = auth.org_id;
    let org_slug = auth.org_slug.clone();

    // Check max_gateways_per_org limit
    let max_gateways = crate::routes::settings::get_org_setting_value(
        &state.db,
        org_id,
        "max_gateways_per_org",
        "50",
    )
    .await
    .parse::<i64>()
    .unwrap_or(50);

    let current_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM gateways WHERE org_id = $1")
        .bind(org_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    if current_count.0 >= max_gateways {
        return Err((
            StatusCode::FORBIDDEN,
            format!(
                "Gateway limit reached. Maximum {} gateways per organization.",
                max_gateways
            ),
        ));
    }

    if !payload
        .slug
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err((StatusCode::BAD_REQUEST, "Invalid slug format".to_string()));
    }

    let row: GatewayRow = sqlx::query_as(SQL_INSERT_GATEWAY)
        .bind(org_id)
        .bind(&payload.name)
        .bind(&payload.slug)
        .bind(payload.enabled)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    for server_name in &payload.servers {
        let _ = sqlx::query(SQL_ADD_SERVER_TO_GATEWAY)
            .bind(row.id)
            .bind(server_name)
            .bind(org_id)
            .execute(&state.db.pool)
            .await;
    }

    // Record audit log
    let _ = crate::services::audit::record_audit_log(
        &state.db.pool,
        org_id,
        Some(auth.user_id),
        crate::services::audit::actions::GATEWAY_CREATE,
        crate::services::audit::resource_types::GATEWAY,
        Some(row.id),
        Some(&row.name),
        Some(serde_json::json!({
            "slug": &row.slug,
            "enabled": row.enabled,
            "servers": &payload.servers
        })),
        None,
        None,
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(GatewayResponse {
            id: row.id,
            name: row.name,
            slug: row.slug.clone(),
            enabled: row.enabled,
            proxy_url: format!("{}/mcp/{}/{}", state.config.base_url, org_slug, row.slug),
            servers: vec![],
            created_at: row.created_at,
            updated_at: row.updated_at,
        }),
    ))
}

pub async fn update_gateway(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(payload): Json<UpdateGatewayRequest>,
) -> Result<Json<GatewayResponse>, (StatusCode, String)> {
    // Check permission
    auth.require(Permission::GatewaysWrite)?;

    let org_id = auth.org_id;
    let org_slug = auth.org_slug.clone();

    let existing: Option<GatewayRow> =
        sqlx::query_as("SELECT * FROM gateways WHERE slug = $1 AND org_id = $2")
            .bind(&slug)
            .bind(org_id)
            .fetch_optional(&state.db.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("{}: {}", error::DATABASE_ERROR, e),
                )
            })?;

    let existing = existing.ok_or((StatusCode::NOT_FOUND, "Gateway not found".to_string()))?;

    let new_name = payload.name.unwrap_or(existing.name);
    let new_slug = payload.slug.unwrap_or(existing.slug);
    let new_enabled = payload.enabled.unwrap_or(existing.enabled);

    let row: GatewayRow = sqlx::query_as(SQL_UPDATE_GATEWAY)
        .bind(&new_name)
        .bind(&new_slug)
        .bind(new_enabled)
        .bind(existing.id)
        .bind(org_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    // Record audit log
    let _ = crate::services::audit::record_audit_log(
        &state.db.pool,
        org_id,
        Some(auth.user_id),
        crate::services::audit::actions::GATEWAY_UPDATE,
        crate::services::audit::resource_types::GATEWAY,
        Some(row.id),
        Some(&row.name),
        Some(serde_json::json!({
            "slug": &row.slug,
            "enabled": row.enabled
        })),
        None,
        None,
    )
    .await;

    Ok(Json(GatewayResponse {
        id: row.id,
        name: row.name,
        slug: row.slug.clone(),
        enabled: row.enabled,
        proxy_url: format!("{}/mcp/{}/{}", state.config.base_url, org_slug, row.slug),
        servers: vec![],
        created_at: row.created_at,
        updated_at: row.updated_at,
    }))
}

pub async fn delete_gateway(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Check permission
    auth.require(Permission::GatewaysWrite)?;

    let org_id = auth.org_id;

    sqlx::query(SQL_DELETE_GATEWAY)
        .bind(&slug)
        .bind(org_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    // Record audit log
    let _ = crate::services::audit::record_audit_log(
        &state.db.pool,
        org_id,
        Some(auth.user_id),
        crate::services::audit::actions::GATEWAY_DELETE,
        crate::services::audit::resource_types::GATEWAY,
        None,
        Some(&slug),
        None,
        None,
        None,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_server_to_gateway(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(payload): Json<AddServerRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Check permission
    auth.require(Permission::GatewaysWrite)?;

    let org_id = auth.org_id;

    let gateway_id: Option<(Uuid,)> = sqlx::query_as(SQL_GET_GATEWAY_ID)
        .bind(&slug)
        .bind(org_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let gateway_id = gateway_id
        .ok_or((StatusCode::NOT_FOUND, "Gateway not found".to_string()))?
        .0;

    sqlx::query(SQL_ADD_SERVER_TO_GATEWAY)
        .bind(gateway_id)
        .bind(&payload.server_name)
        .bind(org_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    Ok(StatusCode::CREATED)
}

pub async fn remove_server_from_gateway(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((slug, server_name)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Check permission
    auth.require(Permission::GatewaysWrite)?;

    let org_id = auth.org_id;

    let gateway_id: Option<(Uuid,)> = sqlx::query_as(SQL_GET_GATEWAY_ID)
        .bind(&slug)
        .bind(org_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let gateway_id = gateway_id
        .ok_or((StatusCode::NOT_FOUND, "Gateway not found".to_string()))?
        .0;

    sqlx::query(SQL_REMOVE_SERVER_FROM_GATEWAY)
        .bind(gateway_id)
        .bind(&server_name)
        .bind(org_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// LIST GATEWAY TOOLS (Admin Endpoint)
// ============================================

#[derive(Debug, Serialize)]
pub struct GatewayToolResponse {
    pub name: String,
    pub description: Option<String>,
}

const SQL_GET_GATEWAY_SERVERS: &str = r#"
    SELECT s.id, s.name, s.url, s.auth_type, s.oauth_client_id, s.oauth_token_url
    FROM gateway_servers gs
    JOIN servers s ON s.id = gs.server_id
    WHERE gs.gateway_id = $1
    ORDER BY gs.priority
"#;

const SQL_SELECT_GOVERNANCE: &str = r#"
    SELECT allowed_tools, denied_tools, tool_prefix FROM governance_configs WHERE server_id = $1
"#;

#[derive(Debug, sqlx::FromRow)]
struct GovernanceRow {
    allowed_tools: serde_json::Value,
    denied_tools: serde_json::Value,
    tool_prefix: String,
}

#[derive(Default)]
struct GovernanceConfig {
    allowed_tools: Vec<String>,
    denied_tools: Vec<String>,
    tool_prefix: String,
}

impl GovernanceConfig {
    fn is_tool_allowed(&self, name: &str) -> bool {
        // If allowlist defined, tool must be in it
        if !self.allowed_tools.is_empty() && !self.allowed_tools.contains(&name.to_string()) {
            return false;
        }
        // Tool must not be in denylist
        !self.denied_tools.contains(&name.to_string())
    }

    fn add_prefix(&self, name: &str) -> String {
        if self.tool_prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}_{}", self.tool_prefix, name)
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ServerRowForTools {
    id: Uuid,
    name: String,
    url: String,
    auth_type: Option<String>,
    oauth_client_id: Option<String>,
    oauth_token_url: Option<String>,
}

impl crate::services::server_auth::AuthenticableServer for ServerRowForTools {
    fn id(&self) -> Uuid {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn auth_type(&self) -> Option<&str> {
        self.auth_type.as_deref()
    }
    fn oauth_client_id(&self) -> Option<&str> {
        self.oauth_client_id.as_deref()
    }
    fn oauth_token_url(&self) -> Option<&str> {
        self.oauth_token_url.as_deref()
    }
}

/// GET /api/gateways/:slug/tools - List tools from all servers in gateway (JWT auth)
/// Fetches tools from each server and applies governance (tool prefix, allowlist, blocklist)
pub async fn list_gateway_tools(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<Vec<GatewayToolResponse>>, (StatusCode, String)> {
    let org_id = auth.org_id;

    // Get gateway ID
    let gateway_id: Option<(Uuid,)> = sqlx::query_as(SQL_GET_GATEWAY_ID)
        .bind(&slug)
        .bind(org_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let gateway_id = gateway_id
        .ok_or((StatusCode::NOT_FOUND, "Gateway not found".to_string()))?
        .0;

    // Get all servers in gateway
    let servers: Vec<ServerRowForTools> = sqlx::query_as(SQL_GET_GATEWAY_SERVERS)
        .bind(gateway_id)
        .fetch_all(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    if servers.is_empty() {
        return Ok(Json(vec![]));
    }

    // Aggregate tools from all servers
    let mut all_tools: Vec<GatewayToolResponse> = Vec::new();

    for server in &servers {
        // Build auth headers for server using centralized service
        let (auth_header_name, auth_header_value) = crate::services::server_auth::get_auth_headers(
            &state.db.pool,
            server,
            &state.config.encryption_key,
            Some(org_id),
        )
        .await;

        // Fetch governance config for this server
        let governance = get_server_governance(&state, server.id).await;

        // Fetch tools from upstream server
        match crate::services::mcp_client::list_tools_from_server(
            &server.url,
            auth_header_name.as_deref(),
            auth_header_value.as_deref(),
        )
        .await
        {
            Ok(tools) => {
                for tool in tools {
                    // Apply governance filtering
                    if governance.is_tool_allowed(&tool.name) {
                        // Apply prefix from governance
                        let prefixed_name = governance.add_prefix(&tool.name);
                        all_tools.push(GatewayToolResponse {
                            name: prefixed_name,
                            description: tool.description,
                        });
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch tools from server {}: {}", server.name, e);
            }
        }
    }

    Ok(Json(all_tools))
}

/// Fetch governance config for a server
async fn get_server_governance(state: &AppState, server_id: Uuid) -> GovernanceConfig {
    let row: Option<GovernanceRow> = sqlx::query_as(SQL_SELECT_GOVERNANCE)
        .bind(server_id)
        .fetch_optional(&state.db.pool)
        .await
        .ok()
        .flatten();

    match row {
        Some(r) => GovernanceConfig {
            allowed_tools: json_to_vec(&r.allowed_tools),
            denied_tools: json_to_vec(&r.denied_tools),
            tool_prefix: r.tool_prefix,
        },
        None => GovernanceConfig::default(),
    }
}

fn json_to_vec(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
