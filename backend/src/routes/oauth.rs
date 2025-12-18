use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::auth::AuthUser;
use crate::services::{crypto, oauth_client};
use crate::AppState;

// --- DTOs ---

#[derive(Debug, Deserialize)]
pub struct OAuthConfigRequest {
    pub authorization_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub scopes: Option<String>,
    pub use_pkce: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct OAuthAuthorizeResponse {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthStatusResponse {
    pub connected: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

// --- Handlers ---

/// Configure OAuth settings for a server
pub async fn configure_oauth(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    auth_user: AuthUser,
    Json(req): Json<OAuthConfigRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Verify server ownership
    let server_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM servers WHERE name = $1 AND user_id = $2"
    )
    .bind(&server_name)
    .bind(&auth_user.user_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Server not found".to_string()))?;

    // Encrypt client_secret if provided
    let encrypted_secret = req.client_secret.map(|s| {
        let key = crypto::derive_key(&state.config.encryption_key);
        crypto::encrypt(&s, &key).unwrap_or_default()
    });

    // Update server with OAuth config
    sqlx::query(
        "UPDATE servers SET 
            auth_type = 'oauth',
            oauth_authorization_url = $1,
            oauth_token_url = $2,
            oauth_client_id = $3,
            oauth_scopes = $4,
            oauth_use_pkce = $5
         WHERE id = $6"
    )
    .bind(&req.authorization_url)
    .bind(&req.token_url)
    .bind(&req.client_id)
    .bind(&req.scopes)
    .bind(req.use_pkce.unwrap_or(true))
    .bind(&server_id)
    .execute(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Store encrypted client_secret in credentials if provided
    if let Some(secret) = encrypted_secret {
        // Delete existing oauth_client_secret credential
        sqlx::query("DELETE FROM credentials WHERE server_id = $1 AND credential_type = 'oauth_client_secret'")
            .bind(&server_id)
            .execute(&state.db.pool)
            .await
            .ok();

        sqlx::query(
            "INSERT INTO credentials (server_id, credential_type, encrypted_value, name)
             VALUES ($1, 'oauth_client_secret', $2, 'OAuth Client Secret')"
        )
        .bind(&server_id)
        .bind(&secret)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(StatusCode::OK)
}

/// Start OAuth authorization flow - returns URL for user to authorize
pub async fn start_oauth(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    auth_user: AuthUser,
) -> Result<Json<OAuthAuthorizeResponse>, (StatusCode, String)> {
    // Get server with OAuth config
    let server = sqlx::query_as::<_, oauth_client::ServerOAuthConfig>(
        "SELECT id, oauth_authorization_url, oauth_token_url, oauth_client_id, 
                oauth_scopes, oauth_use_pkce 
         FROM servers WHERE name = $1 AND user_id = $2 AND auth_type = 'oauth'"
    )
    .bind(&server_name)
    .bind(&auth_user.user_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Server not found or not configured for OAuth".to_string()))?;

    // Generate authorization URL with PKCE if enabled
    let callback_url = format!("{}/api/servers/{}/oauth/callback", 
        state.config.base_url, server_name);
    
    let (auth_url, oauth_state, code_verifier) = oauth_client::generate_authorization_url(
        &server,
        &callback_url,
    );

    // Store state and code_verifier temporarily
    sqlx::query(
        "INSERT INTO oauth_tokens (server_id, user_id, access_token_encrypted, oauth_state, pkce_code_verifier)
         VALUES ($1, $2, '', $3, $4)
         ON CONFLICT (server_id, user_id) 
         DO UPDATE SET oauth_state = $3, pkce_code_verifier = $4, updated_at = NOW()"
    )
    .bind(&server.id)
    .bind(&auth_user.user_id)
    .bind(&oauth_state)
    .bind(&code_verifier)
    .execute(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(OAuthAuthorizeResponse {
        authorization_url: auth_url,
        state: oauth_state,
    }))
}

/// OAuth callback - exchanges code for tokens
pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Redirect, (StatusCode, String)> {
    // Find the pending OAuth flow by state
    let pending = sqlx::query_as::<_, oauth_client::PendingOAuth>(
        "SELECT ot.id, ot.server_id, ot.user_id, ot.pkce_code_verifier, 
                s.oauth_token_url, s.oauth_client_id, s.oauth_use_pkce
         FROM oauth_tokens ot
         JOIN servers s ON s.id = ot.server_id
         WHERE s.name = $1 AND ot.oauth_state = $2"
    )
    .bind(&server_name)
    .bind(&query.state)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()))?;

    // Get client_secret if exists
    let client_secret: Option<String> = sqlx::query_scalar(
        "SELECT encrypted_value FROM credentials 
         WHERE server_id = $1 AND credential_type = 'oauth_client_secret'"
    )
    .bind(&pending.server_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(|encrypted: String| {
        let key = crypto::derive_key(&state.config.encryption_key);
        crypto::decrypt(&encrypted, &key).unwrap_or_default()
    });

    let callback_url = format!("{}/api/servers/{}/oauth/callback", 
        state.config.base_url, server_name);

    // Exchange code for tokens
    let tokens = oauth_client::exchange_code(
        &pending,
        &query.code,
        &callback_url,
        client_secret.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Encrypt and store tokens
    let key = crypto::derive_key(&state.config.encryption_key);
    let access_encrypted = crypto::encrypt(&tokens.access_token, &key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let refresh_encrypted = tokens.refresh_token
        .map(|r| crypto::encrypt(&r, &key).unwrap_or_default());

    sqlx::query(
        "UPDATE oauth_tokens SET 
            access_token_encrypted = $1,
            refresh_token_encrypted = $2,
            token_type = $3,
            expires_at = $4,
            scope = $5,
            oauth_state = NULL,
            pkce_code_verifier = NULL,
            updated_at = NOW()
         WHERE id = $6"
    )
    .bind(&access_encrypted)
    .bind(&refresh_encrypted)
    .bind(&tokens.token_type)
    .bind(&tokens.expires_at)
    .bind(&tokens.scope)
    .bind(&pending.id)
    .execute(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Redirect to frontend with success
    let redirect_url = format!("{}/servers/{}?oauth=success", 
        state.config.frontend_url, server_name);
    
    Ok(Redirect::temporary(&redirect_url))
}

/// Get OAuth connection status
pub async fn oauth_status(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    auth_user: AuthUser,
) -> Result<Json<OAuthStatusResponse>, (StatusCode, String)> {
    let result = sqlx::query_as::<_, (Option<chrono::DateTime<chrono::Utc>>, Option<String>)>(
        "SELECT ot.expires_at, ot.scope
         FROM oauth_tokens ot
         JOIN servers s ON s.id = ot.server_id
         WHERE s.name = $1 AND s.user_id = $2 AND ot.user_id = $2
         AND ot.access_token_encrypted != ''"
    )
    .bind(&server_name)
    .bind(&auth_user.user_id)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match result {
        Some((expires_at, scope)) => Ok(Json(OAuthStatusResponse {
            connected: true,
            expires_at,
            scope,
        })),
        None => Ok(Json(OAuthStatusResponse {
            connected: false,
            expires_at: None,
            scope: None,
        })),
    }
}

/// Revoke OAuth connection
pub async fn revoke_oauth(
    State(state): State<Arc<AppState>>,
    Path(server_name): Path<String>,
    auth_user: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query(
        "DELETE FROM oauth_tokens ot
         USING servers s
         WHERE ot.server_id = s.id AND s.name = $1 AND s.user_id = $2 AND ot.user_id = $2"
    )
    .bind(&server_name)
    .bind(&auth_user.user_id)
    .execute(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
