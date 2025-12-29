//! Repository traits for database abstraction
//!
//! This module provides trait-based abstractions for database operations,
//! enabling mocking in tests without requiring a real PostgreSQL connection.

use async_trait::async_trait;
use uuid::Uuid;

/// Result type for repository operations
pub type RepoResult<T> = Result<T, RepoError>;

/// Error type for repository operations
#[derive(Debug)]
pub enum RepoError {
    NotFound,
    DatabaseError(String),
    ValidationError(String),
}

impl From<sqlx::Error> for RepoError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => RepoError::NotFound,
            _ => RepoError::DatabaseError(e.to_string()),
        }
    }
}

// ============================================
// PAT REPOSITORY
// ============================================

/// Personal Access Token data for validation
#[derive(Debug, Clone)]
pub struct PatData {
    pub id: Uuid,
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Repository trait for PAT operations
#[async_trait]
pub trait PatRepository: Send + Sync {
    async fn find_by_hash(&self, token_hash: &str) -> RepoResult<Option<PatData>>;
    async fn update_last_used(&self, id: Uuid) -> RepoResult<()>;
}

// ============================================
// SERVICE ACCOUNT REPOSITORY
// ============================================

/// Service Account data for validation
#[derive(Debug, Clone)]
pub struct ServiceAccountData {
    pub id: Uuid,
    pub org_id: Uuid,
    pub scopes: Vec<String>,
}

/// Repository trait for Service Account operations
#[async_trait]
pub trait ServiceAccountRepository: Send + Sync {
    async fn validate_credentials(
        &self,
        client_id: &str,
        secret_hash: &str,
    ) -> RepoResult<Option<ServiceAccountData>>;
    async fn update_last_used(&self, id: Uuid) -> RepoResult<()>;
}

// ============================================
// POSTGRES IMPLEMENTATIONS
// ============================================

use sqlx::PgPool;

/// PostgreSQL implementation of PatRepository
pub struct PgPatRepository {
    pool: PgPool,
}

impl PgPatRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PatRepository for PgPatRepository {
    async fn find_by_hash(&self, token_hash: &str) -> RepoResult<Option<PatData>> {
        let row: Option<(Uuid, Uuid, Uuid, String, sqlx::types::Json<Vec<String>>, Option<chrono::DateTime<chrono::Utc>>)> = 
            sqlx::query_as(
                r#"SELECT id, user_id, org_id, name, scopes, expires_at
                   FROM personal_access_tokens
                   WHERE token_hash = $1"#
            )
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|(id, user_id, org_id, name, scopes, expires_at)| PatData {
            id,
            user_id,
            org_id,
            name,
            scopes: scopes.0,
            expires_at,
        }))
    }

    async fn update_last_used(&self, id: Uuid) -> RepoResult<()> {
        sqlx::query("UPDATE personal_access_tokens SET last_used_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// PostgreSQL implementation of ServiceAccountRepository
pub struct PgServiceAccountRepository {
    pool: PgPool,
}

impl PgServiceAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ServiceAccountRepository for PgServiceAccountRepository {
    async fn validate_credentials(
        &self,
        client_id: &str,
        secret_hash: &str,
    ) -> RepoResult<Option<ServiceAccountData>> {
        let row: Option<(Uuid, Uuid, serde_json::Value)> = sqlx::query_as(
            r#"SELECT id, org_id, scopes
               FROM service_accounts
               WHERE client_id = $1 AND client_secret_hash = $2 AND enabled = true"#
        )
        .bind(client_id)
        .bind(secret_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, org_id, scopes)| ServiceAccountData {
            id,
            org_id,
            scopes: serde_json::from_value(scopes).unwrap_or_default(),
        }))
    }

    async fn update_last_used(&self, id: Uuid) -> RepoResult<()> {
        sqlx::query("UPDATE service_accounts SET last_used_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// ============================================
// MOCK IMPLEMENTATIONS (for tests)
// ============================================

#[cfg(test)]
pub mod mocks {
    use super::*;
    use std::sync::Mutex;

    /// Mock PAT Repository for testing
    pub struct MockPatRepository {
        pub find_result: Mutex<Option<PatData>>,
    }

    impl MockPatRepository {
        pub fn new() -> Self {
            Self {
                find_result: Mutex::new(None),
            }
        }

        pub fn with_pat(pat: PatData) -> Self {
            Self {
                find_result: Mutex::new(Some(pat)),
            }
        }
    }

    #[async_trait]
    impl PatRepository for MockPatRepository {
        async fn find_by_hash(&self, _token_hash: &str) -> RepoResult<Option<PatData>> {
            Ok(self.find_result.lock().unwrap().clone())
        }

        async fn update_last_used(&self, _id: Uuid) -> RepoResult<()> {
            Ok(())
        }
    }

    /// Mock Service Account Repository for testing
    pub struct MockServiceAccountRepository {
        pub validate_result: Mutex<Option<ServiceAccountData>>,
    }

    impl MockServiceAccountRepository {
        pub fn new() -> Self {
            Self {
                validate_result: Mutex::new(None),
            }
        }

        pub fn with_account(account: ServiceAccountData) -> Self {
            Self {
                validate_result: Mutex::new(Some(account)),
            }
        }
    }

    #[async_trait]
    impl ServiceAccountRepository for MockServiceAccountRepository {
        async fn validate_credentials(
            &self,
            _client_id: &str,
            _secret_hash: &str,
        ) -> RepoResult<Option<ServiceAccountData>> {
            Ok(self.validate_result.lock().unwrap().clone())
        }

        async fn update_last_used(&self, _id: Uuid) -> RepoResult<()> {
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn mock_pat_returns_configured_data() {
            let pat = PatData {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                org_id: Uuid::new_v4(),
                name: "Test PAT".to_string(),
                scopes: vec!["mcp:server:read".to_string()],
                expires_at: None,
            };
            
            let repo = MockPatRepository::with_pat(pat.clone());
            let result = repo.find_by_hash("any-hash").await.unwrap();
            
            assert!(result.is_some());
            assert_eq!(result.unwrap().name, "Test PAT");
        }

        #[tokio::test]
        async fn mock_pat_returns_none_by_default() {
            let repo = MockPatRepository::new();
            let result = repo.find_by_hash("any-hash").await.unwrap();
            assert!(result.is_none());
        }

        #[tokio::test]
        async fn mock_service_account_returns_configured_data() {
            let account = ServiceAccountData {
                id: Uuid::new_v4(),
                org_id: Uuid::new_v4(),
                scopes: vec!["mcp:tool:execute".to_string()],
            };
            
            let repo = MockServiceAccountRepository::with_account(account.clone());
            let result = repo.validate_credentials("client", "hash").await.unwrap();
            
            assert!(result.is_some());
            assert_eq!(result.unwrap().scopes.len(), 1);
        }
    }
}
