//! Service Account (M2M) API Routes
//!
//! Endpoints for creating, listing, and managing Service Accounts
//! for OAuth 2.0 Client Credentials flow.

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
pub struct ServiceAccountResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub client_id: String,
    pub scopes: serde_json::Value,
    pub enabled: bool,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ServiceAccountCreatedResponse {
    pub id: Uuid,
    pub name: String,
    pub client_id: String,
    pub client_secret: String,  // Only shown once!
    pub scopes: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceAccountRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
}

fn default_scopes() -> Vec<String> {
    vec!["mcp:tool:execute".to_string()]
}

#[derive(Debug, Deserialize)]
pub struct UpdateServiceAccountRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct ServiceAccountRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    client_id: String,
    scopes: serde_json::Value,
    enabled: bool,
    last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

// ============================================
// SQL QUERIES
// ============================================

const SQL_LIST_SERVICE_ACCOUNTS: &str = r#"
    SELECT id, name, description, client_id, scopes, enabled, last_used_at, created_at
    FROM service_accounts
    WHERE org_id = $1
    ORDER BY created_at DESC
"#;

const SQL_GET_SERVICE_ACCOUNT: &str = r#"
    SELECT id, name, description, client_id, scopes, enabled, last_used_at, created_at
    FROM service_accounts
    WHERE id = $1 AND org_id = $2
"#;

const SQL_INSERT_SERVICE_ACCOUNT: &str = r#"
    INSERT INTO service_accounts (org_id, name, description, client_id, client_secret_hash, scopes, created_by)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    RETURNING id, name, client_id, scopes
"#;

const SQL_UPDATE_SERVICE_ACCOUNT: &str = r#"
    UPDATE service_accounts
    SET name = COALESCE($1, name),
        description = COALESCE($2, description),
        scopes = COALESCE($3, scopes),
        enabled = COALESCE($4, enabled),
        updated_at = NOW()
    WHERE id = $5 AND org_id = $6
    RETURNING id
"#;

const SQL_DELETE_SERVICE_ACCOUNT: &str = r#"
    DELETE FROM service_accounts WHERE id = $1 AND org_id = $2
"#;

// ============================================
// TOKEN GENERATION
// ============================================

const CLIENT_ID_PREFIX: &str = "mcpx_sa_";
const SECRET_PREFIX: &str = "mcpx_secret_";
const ID_RANDOM_LENGTH: usize = 32;
const SECRET_RANDOM_LENGTH: usize = 48;

fn generate_client_id() -> String {
    use rand::Rng;
    let random: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(ID_RANDOM_LENGTH)
        .map(char::from)
        .collect();
    format!("{}{}", CLIENT_ID_PREFIX, random)
}

fn generate_client_secret() -> String {
    use rand::Rng;
    let random: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(SECRET_RANDOM_LENGTH)
        .map(char::from)
        .collect();
    format!("{}{}", SECRET_PREFIX, random)
}

fn hash_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ============================================
// ROUTE HANDLERS
// ============================================

/// GET /api/service-accounts - List org's service accounts
pub async fn list_service_accounts(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Vec<ServiceAccountResponse>>, (StatusCode, String)> {
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let rows: Vec<ServiceAccountRow> = sqlx::query_as(SQL_LIST_SERVICE_ACCOUNTS)
        .bind(org_id)
        .fetch_all(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    let accounts: Vec<ServiceAccountResponse> = rows
        .into_iter()
        .map(|row| ServiceAccountResponse {
            id: row.id,
            name: row.name,
            description: row.description,
            client_id: row.client_id,
            scopes: row.scopes,
            enabled: row.enabled,
            last_used_at: row.last_used_at,
            created_at: row.created_at,
        })
        .collect();

    Ok(Json(accounts))
}

/// POST /api/service-accounts - Create new service account
pub async fn create_service_account(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<CreateServiceAccountRequest>,
) -> Result<Json<ServiceAccountCreatedResponse>, (StatusCode, String)> {
    let (user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    // Validate name
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name is required".to_string()));
    }

    // Generate credentials
    let client_id = generate_client_id();
    let client_secret = generate_client_secret();
    let secret_hash = hash_secret(&client_secret);
    let scopes_json = serde_json::to_value(&payload.scopes).unwrap_or_default();

    // Insert into database
    let row: (Uuid, String, String, serde_json::Value) = sqlx::query_as(SQL_INSERT_SERVICE_ACCOUNT)
        .bind(org_id)
        .bind(payload.name.trim())
        .bind(&payload.description)
        .bind(&client_id)
        .bind(&secret_hash)
        .bind(&scopes_json)
        .bind(user_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    tracing::info!("Service account created: {} ({})", row.1, client_id);

    // Return with secret (shown only once!)
    Ok(Json(ServiceAccountCreatedResponse {
        id: row.0,
        name: row.1,
        client_id: row.2,
        client_secret,  // This is the only time it's visible!
        scopes: row.3,
    }))
}

/// GET /api/service-accounts/:id - Get service account details
pub async fn get_service_account(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(account_id): Path<Uuid>,
) -> Result<Json<ServiceAccountResponse>, (StatusCode, String)> {
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let row: Option<ServiceAccountRow> = sqlx::query_as(SQL_GET_SERVICE_ACCOUNT)
        .bind(account_id)
        .bind(org_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    let row = row.ok_or((StatusCode::NOT_FOUND, "Service account not found".to_string()))?;

    Ok(Json(ServiceAccountResponse {
        id: row.id,
        name: row.name,
        description: row.description,
        client_id: row.client_id,
        scopes: row.scopes,
        enabled: row.enabled,
        last_used_at: row.last_used_at,
        created_at: row.created_at,
    }))
}

/// PUT /api/service-accounts/:id - Update service account
pub async fn update_service_account(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<UpdateServiceAccountRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let scopes_json = payload.scopes.map(|s| serde_json::to_value(s).unwrap_or_default());

    let result = sqlx::query(SQL_UPDATE_SERVICE_ACCOUNT)
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&scopes_json)
        .bind(payload.enabled)
        .bind(account_id)
        .bind(org_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Service account not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/service-accounts/:id - Delete service account
pub async fn delete_service_account(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(account_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let (_user_id, org_id, _org_slug) = extract_org_context(&headers, &state.config.jwt_secret)?;

    let result = sqlx::query(SQL_DELETE_SERVICE_ACCOUNT)
        .bind(account_id)
        .bind(org_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Service account not found".to_string()));
    }

    tracing::info!("Service account deleted: {}", account_id);
    Ok(StatusCode::NO_CONTENT)
}

// ============================================
// SERVICE ACCOUNT VALIDATION (for proxy)
// ============================================

#[derive(Debug, sqlx::FromRow)]
pub struct ServiceAccountValidationResult {
    pub id: Uuid,
    pub org_id: Uuid,
    pub scopes: serde_json::Value,
}

const SQL_VALIDATE_SERVICE_ACCOUNT: &str = r#"
    SELECT id, org_id, scopes
    FROM service_accounts
    WHERE client_id = $1 AND client_secret_hash = $2 AND enabled = true
"#;

/// Validate service account credentials and return account info if valid
pub async fn validate_service_account(
    pool: &sqlx::PgPool,
    client_id: &str,
    client_secret: &str,
) -> Result<Option<ServiceAccountValidationResult>, sqlx::Error> {
    let secret_hash = hash_secret(client_secret);
    
    let result: Option<ServiceAccountValidationResult> = sqlx::query_as(SQL_VALIDATE_SERVICE_ACCOUNT)
        .bind(client_id)
        .bind(&secret_hash)
        .fetch_optional(pool)
        .await?;

    // Update last_used_at if found
    if let Some(ref account) = result {
        let _ = sqlx::query("UPDATE service_accounts SET last_used_at = NOW() WHERE id = $1")
            .bind(account.id)
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
    fn test_generate_client_id_has_correct_prefix() {
        let client_id = generate_client_id();
        assert!(client_id.starts_with(CLIENT_ID_PREFIX));
    }

    #[test]
    fn test_generate_client_id_has_correct_length() {
        let client_id = generate_client_id();
        let expected_length = CLIENT_ID_PREFIX.len() + ID_RANDOM_LENGTH;
        assert_eq!(client_id.len(), expected_length);
    }

    #[test]
    fn test_generate_client_id_is_unique() {
        let id1 = generate_client_id();
        let id2 = generate_client_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_client_secret_has_correct_prefix() {
        let secret = generate_client_secret();
        assert!(secret.starts_with(SECRET_PREFIX));
    }

    #[test]
    fn test_generate_client_secret_has_correct_length() {
        let secret = generate_client_secret();
        let expected_length = SECRET_PREFIX.len() + SECRET_RANDOM_LENGTH;
        assert_eq!(secret.len(), expected_length);
    }

    #[test]
    fn test_generate_client_secret_is_unique() {
        let secret1 = generate_client_secret();
        let secret2 = generate_client_secret();
        assert_ne!(secret1, secret2);
    }

    #[test]
    fn test_hash_secret_is_deterministic() {
        let secret = "mcpx_secret_test123456789abcdefgh";
        let hash1 = hash_secret(secret);
        let hash2 = hash_secret(secret);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_secret_different_for_different_secrets() {
        let hash1 = hash_secret("secret1");
        let hash2 = hash_secret("secret2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_secret_is_sha256_hex() {
        let hash = hash_secret("test");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_default_scopes() {
        let scopes = default_scopes();
        assert_eq!(scopes, vec!["mcp:tool:execute"]);
    }
}

// ============================================
// REPOSITORY-BASED VALIDATION (for testing)
// ============================================

use crate::services::repositories::{ServiceAccountRepository, ServiceAccountData};

/// Validate service account credentials using repository trait (testable)
#[allow(dead_code)]
pub async fn validate_credentials_with_repo<R: ServiceAccountRepository>(
    repo: &R,
    client_id: &str,
    client_secret: &str,
) -> Result<Option<ServiceAccountData>, crate::services::repositories::RepoError> {
    let secret_hash = hash_secret(client_secret);
    let result = repo.validate_credentials(client_id, &secret_hash).await?;

    if let Some(ref account) = result {
        // Update last_used_at (ignore errors)
        let _ = repo.update_last_used(account.id).await;
    }

    Ok(result)
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use crate::services::repositories::mocks::MockServiceAccountRepository;

    #[tokio::test]
    async fn validate_returns_none_when_not_found() {
        let repo = MockServiceAccountRepository::new();
        let result = validate_credentials_with_repo(&repo, "client_id", "secret").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn validate_returns_account_when_found() {
        let account = ServiceAccountData {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            scopes: vec!["mcp:server:read".to_string(), "mcp:tool:execute".to_string()],
        };
        let repo = MockServiceAccountRepository::with_account(account.clone());
        
        let result = validate_credentials_with_repo(&repo, "client", "secret").await.unwrap();
        
        assert!(result.is_some());
        let found = result.unwrap();
        assert_eq!(found.scopes.len(), 2);
    }

    #[tokio::test]
    async fn validate_returns_correct_scopes() {
        let account = ServiceAccountData {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            scopes: vec!["mcp:server:read".to_string()],
        };
        let repo = MockServiceAccountRepository::with_account(account);
        
        let result = validate_credentials_with_repo(&repo, "client", "secret").await.unwrap();
        
        assert!(result.is_some());
        let scopes = result.unwrap().scopes;
        assert!(scopes.contains(&"mcp:server:read".to_string()));
        assert!(!scopes.contains(&"mcp:tool:execute".to_string()));
    }
}
