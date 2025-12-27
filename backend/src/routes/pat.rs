//! Personal Access Token (PAT) API Routes
//!
//! Endpoints for creating, listing, and revoking PATs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::routes::auth::extract_org_context;
use crate::messages::error;

// ============================================
// TYPES
// ============================================

#[derive(Debug, Serialize)]
pub struct PatResponse {
    pub id: Uuid,
    pub name: String,
    pub token_prefix: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct PatCreatedResponse {
    pub id: Uuid,
    pub name: String,
    pub token: String,  // Full token - shown only once!
    pub token_prefix: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePatRequest {
    pub name: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct PatRow {
    id: Uuid,
    name: String,
    token_prefix: String,
    scopes: sqlx::types::Json<Vec<String>>,
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ============================================
// SQL QUERIES
// ============================================

const SQL_LIST_PATS: &str = r#"
    SELECT id, name, token_prefix, scopes, last_used_at, expires_at, created_at
    FROM personal_access_tokens
    WHERE user_id = $1 AND org_id = $2
    ORDER BY created_at DESC
"#;

const SQL_CREATE_PAT: &str = r#"
    INSERT INTO personal_access_tokens (user_id, org_id, name, token_hash, token_prefix, scopes, expires_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    RETURNING id, name, token_prefix, scopes, last_used_at, expires_at, created_at
"#;

const SQL_DELETE_PAT: &str = r#"
    DELETE FROM personal_access_tokens
    WHERE id = $1 AND user_id = $2 AND org_id = $3
"#;

const SQL_GET_PAT_BY_HASH: &str = r#"
    SELECT p.id, p.user_id, p.org_id, p.name, p.scopes, p.expires_at
    FROM personal_access_tokens p
    WHERE p.token_hash = $1
"#;

const SQL_UPDATE_LAST_USED: &str = r#"
    UPDATE personal_access_tokens SET last_used_at = NOW() WHERE id = $1
"#;

// ============================================
// TOKEN GENERATION
// ============================================

const TOKEN_PREFIX: &str = "mcpx_pat_";
const TOKEN_RANDOM_LENGTH: usize = 32;

fn generate_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    let random: String = (0..TOKEN_RANDOM_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("{}{}", TOKEN_PREFIX, random)
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_token_prefix(token: &str) -> String {
    token.chars().take(12).collect()
}

// ============================================
// ROUTE HANDLERS
// ============================================

/// GET /api/tokens - List user's tokens for current org
pub async fn list_tokens(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<PatResponse>>, (StatusCode, String)> {
    let (user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let rows = sqlx::query_as::<_, PatRow>(SQL_LIST_PATS)
        .bind(user_id)
        .bind(org_id)
        .fetch_all(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    let response: Vec<PatResponse> = rows
        .into_iter()
        .map(|row| PatResponse {
            id: row.id,
            name: row.name,
            token_prefix: row.token_prefix,
            scopes: row.scopes.0,
            last_used_at: row.last_used_at,
            expires_at: row.expires_at,
            created_at: row.created_at,
        })
        .collect();

    Ok(Json(response))
}

/// POST /api/tokens - Create new token for current org
pub async fn create_token(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreatePatRequest>,
) -> Result<Json<PatCreatedResponse>, (StatusCode, String)> {
    let (user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    // Validate name
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Token name is required".to_string()));
    }

    // Generate token
    let token = generate_token();
    let token_hash = hash_token(&token);
    let token_prefix = get_token_prefix(&token);

    // Calculate expiration
    let expires_at = payload.expires_in_days.map(|days| {
        chrono::Utc::now() + chrono::Duration::days(days)
    });

    // Insert
    let row = sqlx::query_as::<_, PatRow>(SQL_CREATE_PAT)
        .bind(user_id)
        .bind(org_id)
        .bind(&payload.name)
        .bind(&token_hash)
        .bind(&token_prefix)
        .bind(sqlx::types::Json(&payload.scopes))
        .bind(expires_at)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    // Log audit event
    tracing::info!(
        user_id = %user_id,
        token_name = %payload.name,
        "PAT created"
    );

    Ok(Json(PatCreatedResponse {
        id: row.id,
        name: row.name,
        token,  // Full token - shown only once!
        token_prefix: row.token_prefix,
        expires_at: row.expires_at,
        created_at: row.created_at,
    }))
}

/// DELETE /api/tokens/:id - Revoke token
pub async fn delete_token(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(token_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let (user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let result = sqlx::query(SQL_DELETE_PAT)
        .bind(token_id)
        .bind(user_id)
        .bind(org_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Token not found".to_string()));
    }

    tracing::info!(
        user_id = %user_id,
        token_id = %token_id,
        "PAT revoked"
    );

    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// PAT VALIDATION (for proxy)
// ============================================

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)] // scopes will be used for scope validation in future
pub struct PatValidationResult {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub scopes: sqlx::types::Json<Vec<String>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Validate a PAT token and return user info if valid
pub async fn validate_pat(
    pool: &sqlx::PgPool,
    token: &str,
) -> Result<Option<PatValidationResult>, sqlx::Error> {
    // Must start with correct prefix
    if !token.starts_with(TOKEN_PREFIX) {
        return Ok(None);
    }

    let token_hash = hash_token(token);

    let result = sqlx::query_as::<_, PatValidationResult>(SQL_GET_PAT_BY_HASH)
        .bind(&token_hash)
        .fetch_optional(pool)
        .await?;

    if let Some(ref pat) = result {
        // Check expiration
        if let Some(expires_at) = pat.expires_at {
            if expires_at < chrono::Utc::now() {
                return Ok(None);
            }
        }

        // Update last_used_at
        let _ = sqlx::query(SQL_UPDATE_LAST_USED)
            .bind(pat.id)
            .execute(pool)
            .await;
    }

    Ok(result)
}
