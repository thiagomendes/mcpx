//! Server Authentication Service
//!
//! Centralized logic for obtaining authentication headers for MCP servers.
//! Supports: none, api_key, bearer, oauth_auto, oauth_client_credentials

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::crypto;

/// Trait for server types that can be authenticated
pub trait AuthenticableServer {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn auth_type(&self) -> Option<&str>;
    fn oauth_client_id(&self) -> Option<&str>;
    fn oauth_token_url(&self) -> Option<&str>;
}

const SQL_SELECT_CREDENTIAL: &str =
    "SELECT encrypted_value FROM credentials WHERE server_id = $1 AND credential_type = $2";

const SQL_SELECT_OAUTH_TOKEN: &str =
    "SELECT access_token_encrypted, expires_at FROM oauth_tokens WHERE server_id = $1 LIMIT 1";

const SQL_SELECT_OAUTH_TOKEN_BY_ORG: &str =
    "SELECT access_token_encrypted FROM oauth_tokens WHERE server_id = $1 AND org_id = $2";

const SQL_UPSERT_OAUTH_TOKEN: &str = r#"
    INSERT INTO oauth_tokens (server_id, org_id, access_token_encrypted, expires_at, created_at, updated_at)
    VALUES ($1, $2, $3, $4, NOW(), NOW())
    ON CONFLICT (server_id, org_id) 
    DO UPDATE SET access_token_encrypted = $3, expires_at = $4, updated_at = NOW()
"#;

/// Get authentication headers for a server
/// Returns (header_name, header_value) tuple
pub async fn get_auth_headers<S: AuthenticableServer>(
    pool: &PgPool,
    server: &S,
    encryption_key: &str,
    org_id: Option<Uuid>,
) -> (Option<String>, Option<String>) {
    match server.auth_type() {
        Some("api_key") => get_api_key_header(pool, server.id(), encryption_key).await,
        Some("bearer") => get_bearer_header(pool, server.id(), encryption_key).await,
        Some("oauth_auto") => {
            get_oauth_auto_header(pool, server.id(), encryption_key, org_id).await
        }
        Some("oauth_client_credentials") => {
            get_oauth_client_credentials_header(pool, server, encryption_key).await
        }
        _ => (None, None),
    }
}

async fn get_api_key_header(
    pool: &PgPool,
    server_id: Uuid,
    encryption_key: &str,
) -> (Option<String>, Option<String>) {
    let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_CREDENTIAL)
        .bind(server_id)
        .bind("api_key")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    if let Some((encrypted,)) = row {
        let key = crypto::derive_key(encryption_key);
        match crypto::decrypt(&encrypted, &key) {
            Ok(api_key) => (Some("X-API-Key".to_string()), Some(api_key)),
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    }
}

async fn get_bearer_header(
    pool: &PgPool,
    server_id: Uuid,
    encryption_key: &str,
) -> (Option<String>, Option<String>) {
    let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_CREDENTIAL)
        .bind(server_id)
        .bind("bearer")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    if let Some((encrypted,)) = row {
        let key = crypto::derive_key(encryption_key);
        match crypto::decrypt(&encrypted, &key) {
            Ok(token) => (
                Some("Authorization".to_string()),
                Some(format!("Bearer {}", token)),
            ),
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    }
}

async fn get_oauth_auto_header(
    pool: &PgPool,
    server_id: Uuid,
    encryption_key: &str,
    org_id: Option<Uuid>,
) -> (Option<String>, Option<String>) {
    // Try org-specific token first, then fall back to server-level
    let token = if let Some(oid) = org_id {
        let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_OAUTH_TOKEN_BY_ORG)
            .bind(server_id)
            .bind(oid)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

        if let Some((encrypted,)) = row {
            let key = crypto::derive_key(encryption_key);
            crypto::decrypt(&encrypted, &key).ok()
        } else {
            None
        }
    } else {
        // Server-level token (for health checks)
        let row: Option<(String, Option<chrono::DateTime<Utc>>)> =
            sqlx::query_as(SQL_SELECT_OAUTH_TOKEN)
                .bind(server_id)
                .fetch_optional(pool)
                .await
                .ok()
                .flatten();

        if let Some((encrypted, _expires)) = row {
            let key = crypto::derive_key(encryption_key);
            crypto::decrypt(&encrypted, &key).ok()
        } else {
            None
        }
    };

    match token {
        Some(t) => (
            Some("Authorization".to_string()),
            Some(format!("Bearer {}", t)),
        ),
        None => (None, None),
    }
}

async fn get_oauth_client_credentials_header<S: AuthenticableServer>(
    pool: &PgPool,
    server: &S,
    encryption_key: &str,
) -> (Option<String>, Option<String>) {
    let client_id = server.oauth_client_id().unwrap_or_default().to_string();
    let token_url = server.oauth_token_url().unwrap_or_default().to_string();

    if client_id.is_empty() || token_url.is_empty() {
        tracing::warn!(
            "Missing OAuth Client Credentials config for server {}",
            server.name()
        );
        return (None, None);
    }

    // 1. Check for existing valid token
    let existing_token: Option<(String, Option<chrono::DateTime<Utc>>)> =
        sqlx::query_as(SQL_SELECT_OAUTH_TOKEN)
            .bind(server.id())
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    if let Some((encrypted, expires_at)) = existing_token {
        // Check if token is still valid (with 60 second buffer)
        let is_valid = expires_at
            .map(|exp| exp > Utc::now() + Duration::seconds(60))
            .unwrap_or(false);

        if is_valid {
            let key = crypto::derive_key(encryption_key);
            if let Ok(token) = crypto::decrypt(&encrypted, &key) {
                return (
                    Some("Authorization".to_string()),
                    Some(format!("Bearer {}", token)),
                );
            }
        }
    }

    // 2. Fetch new token from token endpoint
    let client_secret = get_client_secret(pool, server.id(), encryption_key).await;

    let Some(secret) = client_secret else {
        tracing::warn!("Missing client secret for server {}", server.name());
        return (None, None);
    };

    let client = reqwest::Client::new();
    let token_response = client
        .post(&token_url)
        .json(&serde_json::json!({
            "grant_type": "client_credentials",
            "client_id": client_id,
            "client_secret": secret
        }))
        .send()
        .await;

    match token_response {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
                    // 3. Persist token for future use
                    let expires_in = json
                        .get("expires_in")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(3600);
                    let expires_at = Utc::now() + Duration::seconds(expires_in);

                    let key = crypto::derive_key(encryption_key);
                    if let Ok(encrypted_token) = crypto::encrypt(access_token, &key) {
                        // Use Uuid::nil() as org_id for client_credentials (server-level token)
                        let _ = sqlx::query(SQL_UPSERT_OAUTH_TOKEN)
                            .bind(server.id())
                            .bind(Uuid::nil())
                            .bind(&encrypted_token)
                            .bind(expires_at)
                            .execute(pool)
                            .await;

                        tracing::info!(
                            "OAuth Client Credentials token obtained and persisted for {}",
                            server.name()
                        );
                    }

                    return (
                        Some("Authorization".to_string()),
                        Some(format!("Bearer {}", access_token)),
                    );
                }
            }
        }
        Ok(resp) => {
            tracing::error!(
                "Token endpoint returned error {} for {}",
                resp.status(),
                server.name()
            );
        }
        Err(e) => {
            tracing::error!("Failed to fetch token for {}: {:?}", server.name(), e);
        }
    }

    (None, None)
}

async fn get_client_secret(pool: &PgPool, server_id: Uuid, encryption_key: &str) -> Option<String> {
    let row: Option<(String,)> = sqlx::query_as(SQL_SELECT_CREDENTIAL)
        .bind(server_id)
        .bind("oauth_client_secret")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

    if let Some((encrypted,)) = row {
        let key = crypto::derive_key(encryption_key);
        crypto::decrypt(&encrypted, &key).ok()
    } else {
        None
    }
}
