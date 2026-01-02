#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub url: String,
    pub transport: String,
    pub enabled: bool,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub auth_type: Option<String>,
    pub oauth_client_id: Option<String>,
    pub oauth_authorization_url: Option<String>,
    pub oauth_token_url: Option<String>,
    pub oauth_scopes: Option<String>,
    pub oauth_use_pkce: Option<bool>,

    pub last_health_check: Option<DateTime<Utc>>,
    pub health_error: Option<String>,
    pub health_check_interval_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub url: String,
    #[serde(default = "default_transport")]
    pub transport: String,
    #[serde(default = "default_auth_type")]
    pub auth_type: String,

    pub api_key: Option<String>,
    pub bearer_token: Option<String>,

    pub oauth_authorization_url: Option<String>,
    pub oauth_token_url: Option<String>,
    pub oauth_client_id: Option<String>,
    pub oauth_client_secret: Option<String>,
    pub oauth_scopes: Option<String>,
}

fn default_transport() -> String {
    "streamable-http".to_string()
}

fn default_auth_type() -> String {
    "none".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub transport: Option<String>,
    pub enabled: Option<bool>,
    pub auth_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerResponse {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub transport: String,
    pub enabled: bool,
    pub auth_type: String,
    pub status: String,
    pub last_health_check: Option<DateTime<Utc>>,
    pub health_error: Option<String>,
    pub proxy_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ServerResponse {
    /// Create ServerResponse from Server - uses org_slug for proxy URL
    pub fn from_server(server: Server, org_slug: &str, base_url: &str) -> Self {
        Self {
            id: server.id,
            name: server.name.clone(),
            url: server.url,
            transport: server.transport,
            enabled: server.enabled,
            auth_type: server.auth_type.unwrap_or_else(|| "none".to_string()),
            status: server
                .status
                .unwrap_or_else(|| "pending_health".to_string()),
            last_health_check: server.last_health_check,
            health_error: server.health_error,
            proxy_url: format!("{}/mcp/{}/{}", base_url, org_slug, server.name),
            created_at: server.created_at,
            updated_at: server.updated_at,
        }
    }
}
