//! Personal Access Token (PAT) API Routes
//!
//! Endpoints for creating, listing, and revoking PATs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::messages::error;
use crate::middleware::auth::AuthUser;
use crate::AppState;

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
    pub token: String, // Full token - shown only once!
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
    auth: AuthUser,
) -> Result<Json<Vec<PatResponse>>, (StatusCode, String)> {
    let user_id = auth.user_id;
    let org_id = auth.org_id;

    let rows = sqlx::query_as::<_, PatRow>(SQL_LIST_PATS)
        .bind(user_id)
        .bind(org_id)
        .fetch_all(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

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
    auth: AuthUser,
    Json(payload): Json<CreatePatRequest>,
) -> Result<Json<PatCreatedResponse>, (StatusCode, String)> {
    // PATs are self-service - any authenticated user can create their own tokens
    // No permission check needed - tokens are automatically tied to auth.user_id

    let user_id = auth.user_id;
    let org_id = auth.org_id;

    // Check max_pats_per_user limit
    let max_pats = crate::routes::settings::get_org_setting_value(
        &state.db,
        org_id,
        "max_pats_per_user",
        "10",
    )
    .await
    .parse::<i64>()
    .unwrap_or(10);

    let current_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM personal_access_tokens WHERE user_id = $1 AND org_id = $2",
    )
    .bind(user_id)
    .bind(org_id)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", error::DATABASE_ERROR, e),
        )
    })?;

    if current_count.0 >= max_pats {
        return Err((
            StatusCode::FORBIDDEN,
            format!("Token limit reached. Maximum {} tokens per user.", max_pats),
        ));
    }

    // Validate name
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Token name is required".to_string(),
        ));
    }

    // Generate token
    let token = generate_token();
    let token_hash = hash_token(&token);
    let token_prefix = get_token_prefix(&token);

    // Calculate expiration
    let expires_at = payload
        .expires_in_days
        .map(|days| chrono::Utc::now() + chrono::Duration::days(days));

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
        Some(user_id),
        crate::services::audit::actions::TOKEN_CREATE,
        crate::services::audit::resource_types::TOKEN,
        Some(row.id),
        Some(&payload.name),
        Some(serde_json::json!({
            "scopes": &payload.scopes,
            "expires_in_days": payload.expires_in_days
        })),
        None,
        None,
    )
    .await;

    Ok(Json(PatCreatedResponse {
        id: row.id,
        name: row.name,
        token, // Full token - shown only once!
        token_prefix: row.token_prefix,
        expires_at: row.expires_at,
        created_at: row.created_at,
    }))
}

/// DELETE /api/tokens/:id - Revoke token
pub async fn delete_token(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(token_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    // PATs are self-service - any user can delete their own tokens
    // The SQL query filters by user_id, so users can only delete their own

    let user_id = auth.user_id;
    let org_id = auth.org_id;

    // Fetch token details before deletion for audit log
    let token: Option<PatRow> = sqlx::query_as(
        "SELECT id, name, token_prefix, scopes, last_used_at, expires_at, created_at FROM personal_access_tokens WHERE id = $1 AND user_id = $2 AND org_id = $3"
    )
        .bind(token_id)
        .bind(user_id)
        .bind(org_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    let result = sqlx::query(SQL_DELETE_PAT)
        .bind(token_id)
        .bind(user_id)
        .bind(org_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Token not found".to_string()));
    }

    // Record audit log with full token details
    let _ = crate::services::audit::record_audit_log(
        &state.db.pool,
        org_id,
        Some(user_id),
        crate::services::audit::actions::TOKEN_DELETE,
        crate::services::audit::resource_types::TOKEN,
        Some(token_id),
        token.as_ref().map(|t| t.name.as_str()),
        token.as_ref().map(|t| serde_json::json!({
            "token_prefix": &t.token_prefix,
            "scopes": &t.scopes
        })),
        None,
        None,
    )
    .await;

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

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_has_correct_prefix() {
        let token = generate_token();
        assert!(token.starts_with(TOKEN_PREFIX));
    }

    #[test]
    fn test_generate_token_has_correct_length() {
        let token = generate_token();
        let expected_length = TOKEN_PREFIX.len() + TOKEN_RANDOM_LENGTH;
        assert_eq!(token.len(), expected_length);
    }

    #[test]
    fn test_generate_token_is_unique() {
        let token1 = generate_token();
        let token2 = generate_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_hash_token_is_deterministic() {
        let token = "mcpx_pat_test123456789abcdefghijkl";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_different_for_different_tokens() {
        let hash1 = hash_token("mcpx_pat_token1");
        let hash2 = hash_token("mcpx_pat_token2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_token_is_sha256_hex() {
        let hash = hash_token("test");
        // SHA256 produces 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_get_token_prefix_returns_first_12_chars() {
        let token = "mcpx_pat_abcdefghijklmnop";
        let prefix = get_token_prefix(token);
        assert_eq!(prefix, "mcpx_pat_abc");
        assert_eq!(prefix.len(), 12);
    }

    #[test]
    fn test_get_token_prefix_short_token() {
        let token = "short";
        let prefix = get_token_prefix(token);
        assert_eq!(prefix, "short");
    }
}

// ============================================
// REPOSITORY-BASED VALIDATION (for testing)
// ============================================

use crate::services::repositories::{PatData, PatRepository};

/// Validate a PAT token using the repository trait (testable)
#[allow(dead_code)]
pub async fn validate_pat_with_repo<R: PatRepository>(
    repo: &R,
    token: &str,
) -> Result<Option<PatData>, crate::services::repositories::RepoError> {
    // Must start with correct prefix
    if !token.starts_with(TOKEN_PREFIX) {
        return Ok(None);
    }

    let token_hash = hash_token(token);
    let result = repo.find_by_hash(&token_hash).await?;

    if let Some(ref pat) = result {
        // Check expiration
        if let Some(expires_at) = pat.expires_at {
            if expires_at < chrono::Utc::now() {
                return Ok(None);
            }
        }

        // Update last_used_at (ignore errors)
        let _ = repo.update_last_used(pat.id).await;
    }

    Ok(result)
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::services::repositories::mocks::MockPatRepository;

    #[tokio::test]
    async fn validate_rejects_token_without_prefix() {
        let repo = MockPatRepository::new();
        let result = validate_pat_with_repo(&repo, "invalid_token")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn validate_returns_none_when_not_found() {
        let repo = MockPatRepository::new();
        let token = format!("{}test123", TOKEN_PREFIX);
        let result = validate_pat_with_repo(&repo, &token).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn validate_returns_pat_when_found() {
        let pat = PatData {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            name: "Test PAT".to_string(),
            scopes: vec!["mcp:server:read".to_string()],
            expires_at: None,
        };
        let repo = MockPatRepository::with_pat(pat.clone());

        let token = format!("{}test123", TOKEN_PREFIX);
        let result = validate_pat_with_repo(&repo, &token).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Test PAT");
    }

    #[tokio::test]
    async fn validate_rejects_expired_token() {
        let pat = PatData {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            name: "Expired PAT".to_string(),
            scopes: vec![],
            expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
        };
        let repo = MockPatRepository::with_pat(pat);

        let token = format!("{}test123", TOKEN_PREFIX);
        let result = validate_pat_with_repo(&repo, &token).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn validate_accepts_non_expired_token() {
        let pat = PatData {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            name: "Valid PAT".to_string(),
            scopes: vec!["mcp:tool:execute".to_string()],
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        };
        let repo = MockPatRepository::with_pat(pat);

        let token = format!("{}test123", TOKEN_PREFIX);
        let result = validate_pat_with_repo(&repo, &token).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Valid PAT");
    }
}
