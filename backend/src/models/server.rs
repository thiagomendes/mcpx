use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub url: String,
    pub transport: String,
    pub enabled: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub url: String,
    #[serde(default = "default_transport")]
    pub transport: String,
}

fn default_transport() -> String {
    "streamable-http".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub transport: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerResponse {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub transport: String,
    pub enabled: bool,
    pub proxy_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ServerResponse {
    pub fn from_server(server: Server, user_id: Uuid, base_url: &str) -> Self {
        Self {
            id: server.id,
            name: server.name.clone(),
            url: server.url,
            transport: server.transport,
            enabled: server.enabled,
            proxy_url: format!("{}/mcp/{}/{}", base_url, user_id, server.name),
            created_at: server.created_at,
            updated_at: server.updated_at,
        }
    }
}
