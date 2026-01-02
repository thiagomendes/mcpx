use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::messages::error;
use crate::middleware::auth::AuthUser;
use crate::services::crypto;
use crate::AppState;

const SQL_SELECT_SERVER_ID: &str = "SELECT id FROM servers WHERE name = $1 AND org_id = $2";

const SQL_UPSERT_OAUTH_TOKENS: &str = r#"
    INSERT INTO oauth_tokens (server_id, org_id, access_token_encrypted, refresh_token_encrypted, token_type, expires_at, scope, dynamic_client_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    ON CONFLICT (server_id, org_id) 
    DO UPDATE SET 
        access_token_encrypted = $3,
        refresh_token_encrypted = $4,
        token_type = $5,
        expires_at = $6,
        scope = $7,
        dynamic_client_id = $8,
        oauth_state = NULL,
        updated_at = NOW()
"#;

const SQL_SELECT_OAUTH_STATUS: &str =
    "SELECT COALESCE(access_token_encrypted, ''), expires_at FROM oauth_tokens WHERE server_id = $1 AND org_id = $2";

const SQL_DELETE_OAUTH_TOKENS: &str =
    "DELETE FROM oauth_tokens WHERE server_id = $1 AND org_id = $2";

#[derive(Debug, Deserialize)]
pub struct StoreTokensRequest {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: Option<i64>,
    pub scope: Option<String>,
    pub client_id: Option<String>, // Dynamic client_id from OAuth registration
}

#[derive(Debug, Serialize)]
pub struct OAuthStatusResponse {
    pub connected: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn store_oauth_tokens(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    auth_user: AuthUser,
    Json(tokens): Json<StoreTokensRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let server_id: Option<Uuid> = sqlx::query_scalar(SQL_SELECT_SERVER_ID)
        .bind(&server_name)
        .bind(auth_user.org_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let server_id =
        server_id.ok_or((StatusCode::NOT_FOUND, error::SERVER_NOT_FOUND.to_string()))?;

    let key = crypto::derive_key(&state.config.encryption_key);

    let access_encrypted = crypto::encrypt(&tokens.access_token, &key).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", error::ENCRYPTION_ERROR, e),
        )
    })?;

    let refresh_encrypted = match &tokens.refresh_token {
        Some(rt) => Some(crypto::encrypt(rt, &key).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::ENCRYPTION_ERROR, e),
            )
        })?),
        None => None,
    };

    let expires_at = tokens
        .expires_in
        .map(|secs| Utc::now() + Duration::seconds(secs));

    sqlx::query(SQL_UPSERT_OAUTH_TOKENS)
        .bind(server_id)
        .bind(auth_user.org_id)
        .bind(&access_encrypted)
        .bind(&refresh_encrypted)
        .bind(tokens.token_type.unwrap_or_else(|| "Bearer".to_string()))
        .bind(expires_at)
        .bind(&tokens.scope)
        .bind(&tokens.client_id) // $8: dynamic_client_id
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

pub async fn oauth_status(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    auth_user: AuthUser,
) -> Result<Json<OAuthStatusResponse>, (StatusCode, String)> {
    let server_id: Option<Uuid> = sqlx::query_scalar(SQL_SELECT_SERVER_ID)
        .bind(&server_name)
        .bind(auth_user.org_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let server_id =
        server_id.ok_or((StatusCode::NOT_FOUND, error::SERVER_NOT_FOUND.to_string()))?;

    let row: Option<(String, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as(SQL_SELECT_OAUTH_STATUS)
            .bind(server_id)
            .bind(auth_user.org_id)
            .fetch_optional(&state.db.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("{}: {}", error::DATABASE_ERROR, e),
                )
            })?;

    match row {
        Some((token, expires_at)) if !token.is_empty() => Ok(Json(OAuthStatusResponse {
            connected: true,
            expires_at,
        })),
        _ => Ok(Json(OAuthStatusResponse {
            connected: false,
            expires_at: None,
        })),
    }
}

pub async fn revoke_oauth(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    auth_user: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let server_id: Option<Uuid> = sqlx::query_scalar(SQL_SELECT_SERVER_ID)
        .bind(&server_name)
        .bind(auth_user.org_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let server_id =
        server_id.ok_or((StatusCode::NOT_FOUND, error::SERVER_NOT_FOUND.to_string()))?;

    sqlx::query(SQL_DELETE_OAUTH_TOKENS)
        .bind(server_id)
        .bind(auth_user.org_id)
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
