//! Repository traits for database abstraction
//!
//! This module provides trait-based abstractions for database operations,
//! enabling mocking in tests without requiring a real PostgreSQL connection.
//!
//! Note: Many items are currently only used in tests, hence the allow(dead_code).

#![allow(dead_code)]

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
// ORG REPOSITORY
// ============================================

/// Organization data
#[derive(Debug, Clone)]
pub struct OrgData {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_personal: bool,
}

/// User membership in organization
#[derive(Debug, Clone)]
pub struct OrgMembership {
    pub org: OrgData,
    pub role: String,
}

/// Repository trait for Organization operations
#[async_trait]
pub trait OrgRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<OrgData>>;
    async fn find_by_slug(&self, slug: &str) -> RepoResult<Option<OrgData>>;
    async fn get_user_orgs(&self, user_id: Uuid) -> RepoResult<Vec<OrgMembership>>;
    async fn get_user_role(&self, user_id: Uuid, org_id: Uuid) -> RepoResult<Option<String>>;
}

// ============================================
// SERVER REPOSITORY
// ============================================

/// Server data
#[derive(Debug, Clone)]
pub struct ServerData {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub url: String,
    pub transport: String,
    pub enabled: bool,
    pub auth_type: String,
}

/// Repository trait for Server operations
#[async_trait]
pub trait ServerRepository: Send + Sync {
    async fn find_by_name(&self, org_id: Uuid, name: &str) -> RepoResult<Option<ServerData>>;
    async fn list_by_org(&self, org_id: Uuid) -> RepoResult<Vec<ServerData>>;
    async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ServerData>>;
}

// ============================================
// GATEWAY REPOSITORY
// ============================================

/// Gateway data
#[derive(Debug, Clone)]
pub struct GatewayData {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub slug: String,
    pub enabled: bool,
}

/// Gateway server association
#[derive(Debug, Clone)]
pub struct GatewayServerData {
    pub server_id: Uuid,
    pub server_name: String,
    pub priority: i32,
}

/// Repository trait for Gateway operations
#[async_trait]
pub trait GatewayRepository: Send + Sync {
    async fn find_by_slug(&self, org_id: Uuid, slug: &str) -> RepoResult<Option<GatewayData>>;
    async fn list_by_org(&self, org_id: Uuid) -> RepoResult<Vec<GatewayData>>;
    async fn get_servers(&self, gateway_id: Uuid) -> RepoResult<Vec<GatewayServerData>>;
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

    // ============================================
    // MOCK ORG REPOSITORY
    // ============================================

    /// Mock Org Repository for testing
    pub struct MockOrgRepository {
        pub org: Mutex<Option<OrgData>>,
        pub memberships: Mutex<Vec<OrgMembership>>,
        pub role: Mutex<Option<String>>,
    }

    impl MockOrgRepository {
        pub fn new() -> Self {
            Self {
                org: Mutex::new(None),
                memberships: Mutex::new(vec![]),
                role: Mutex::new(None),
            }
        }

        pub fn with_org(org: OrgData) -> Self {
            Self {
                org: Mutex::new(Some(org)),
                memberships: Mutex::new(vec![]),
                role: Mutex::new(Some("owner".to_string())),
            }
        }

        pub fn with_membership(membership: OrgMembership) -> Self {
            Self {
                org: Mutex::new(Some(membership.org.clone())),
                memberships: Mutex::new(vec![membership.clone()]),
                role: Mutex::new(Some(membership.role)),
            }
        }
    }

    #[async_trait]
    impl OrgRepository for MockOrgRepository {
        async fn find_by_id(&self, _id: Uuid) -> RepoResult<Option<OrgData>> {
            Ok(self.org.lock().unwrap().clone())
        }

        async fn find_by_slug(&self, _slug: &str) -> RepoResult<Option<OrgData>> {
            Ok(self.org.lock().unwrap().clone())
        }

        async fn get_user_orgs(&self, _user_id: Uuid) -> RepoResult<Vec<OrgMembership>> {
            Ok(self.memberships.lock().unwrap().clone())
        }

        async fn get_user_role(&self, _user_id: Uuid, _org_id: Uuid) -> RepoResult<Option<String>> {
            Ok(self.role.lock().unwrap().clone())
        }
    }

    // ============================================
    // MOCK SERVER REPOSITORY
    // ============================================

    /// Mock Server Repository for testing
    pub struct MockServerRepository {
        pub server: Mutex<Option<ServerData>>,
        pub servers: Mutex<Vec<ServerData>>,
    }

    impl MockServerRepository {
        pub fn new() -> Self {
            Self {
                server: Mutex::new(None),
                servers: Mutex::new(vec![]),
            }
        }

        pub fn with_server(server: ServerData) -> Self {
            Self {
                server: Mutex::new(Some(server.clone())),
                servers: Mutex::new(vec![server]),
            }
        }

        pub fn with_servers(servers: Vec<ServerData>) -> Self {
            Self {
                server: Mutex::new(servers.first().cloned()),
                servers: Mutex::new(servers),
            }
        }
    }

    #[async_trait]
    impl ServerRepository for MockServerRepository {
        async fn find_by_name(&self, _org_id: Uuid, _name: &str) -> RepoResult<Option<ServerData>> {
            Ok(self.server.lock().unwrap().clone())
        }

        async fn list_by_org(&self, _org_id: Uuid) -> RepoResult<Vec<ServerData>> {
            Ok(self.servers.lock().unwrap().clone())
        }

        async fn find_by_id(&self, _id: Uuid) -> RepoResult<Option<ServerData>> {
            Ok(self.server.lock().unwrap().clone())
        }
    }

    // ============================================
    // MOCK GATEWAY REPOSITORY
    // ============================================

    /// Mock Gateway Repository for testing
    pub struct MockGatewayRepository {
        pub gateway: Mutex<Option<GatewayData>>,
        pub gateways: Mutex<Vec<GatewayData>>,
        pub servers: Mutex<Vec<GatewayServerData>>,
    }

    impl MockGatewayRepository {
        pub fn new() -> Self {
            Self {
                gateway: Mutex::new(None),
                gateways: Mutex::new(vec![]),
                servers: Mutex::new(vec![]),
            }
        }

        pub fn with_gateway(gateway: GatewayData) -> Self {
            Self {
                gateway: Mutex::new(Some(gateway.clone())),
                gateways: Mutex::new(vec![gateway]),
                servers: Mutex::new(vec![]),
            }
        }

        pub fn with_servers(gateway: GatewayData, servers: Vec<GatewayServerData>) -> Self {
            Self {
                gateway: Mutex::new(Some(gateway.clone())),
                gateways: Mutex::new(vec![gateway]),
                servers: Mutex::new(servers),
            }
        }
    }

    #[async_trait]
    impl GatewayRepository for MockGatewayRepository {
        async fn find_by_slug(&self, _org_id: Uuid, _slug: &str) -> RepoResult<Option<GatewayData>> {
            Ok(self.gateway.lock().unwrap().clone())
        }

        async fn list_by_org(&self, _org_id: Uuid) -> RepoResult<Vec<GatewayData>> {
            Ok(self.gateways.lock().unwrap().clone())
        }

        async fn get_servers(&self, _gateway_id: Uuid) -> RepoResult<Vec<GatewayServerData>> {
            Ok(self.servers.lock().unwrap().clone())
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

        #[tokio::test]
        async fn mock_org_returns_configured_data() {
            let org = OrgData {
                id: Uuid::new_v4(),
                name: "Test Org".to_string(),
                slug: "test-org".to_string(),
                is_personal: false,
            };
            
            let repo = MockOrgRepository::with_org(org.clone());
            let result = repo.find_by_id(org.id).await.unwrap();
            
            assert!(result.is_some());
            assert_eq!(result.unwrap().name, "Test Org");
        }

        #[tokio::test]
        async fn mock_org_returns_user_role() {
            let org = OrgData {
                id: Uuid::new_v4(),
                name: "Test Org".to_string(),
                slug: "test-org".to_string(),
                is_personal: false,
            };
            let membership = OrgMembership {
                org,
                role: "admin".to_string(),
            };
            
            let repo = MockOrgRepository::with_membership(membership);
            let result = repo.get_user_role(Uuid::new_v4(), Uuid::new_v4()).await.unwrap();
            
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "admin");
        }

        #[tokio::test]
        async fn mock_server_returns_configured_data() {
            let server = ServerData {
                id: Uuid::new_v4(),
                org_id: Uuid::new_v4(),
                name: "test-server".to_string(),
                url: "http://localhost:3000".to_string(),
                transport: "http".to_string(),
                enabled: true,
                auth_type: "none".to_string(),
            };
            
            let repo = MockServerRepository::with_server(server.clone());
            let result = repo.find_by_name(server.org_id, "test-server").await.unwrap();
            
            assert!(result.is_some());
            assert_eq!(result.unwrap().name, "test-server");
        }

        #[tokio::test]
        async fn mock_server_lists_servers() {
            let server1 = ServerData {
                id: Uuid::new_v4(),
                org_id: Uuid::new_v4(),
                name: "server-1".to_string(),
                url: "http://localhost:3000".to_string(),
                transport: "http".to_string(),
                enabled: true,
                auth_type: "none".to_string(),
            };
            let server2 = ServerData {
                id: Uuid::new_v4(),
                org_id: server1.org_id,
                name: "server-2".to_string(),
                url: "http://localhost:4000".to_string(),
                transport: "sse".to_string(),
                enabled: true,
                auth_type: "bearer".to_string(),
            };
            
            let repo = MockServerRepository::with_servers(vec![server1.clone(), server2]);
            let result = repo.list_by_org(server1.org_id).await.unwrap();
            
            assert_eq!(result.len(), 2);
        }

        #[tokio::test]
        async fn mock_gateway_returns_configured_data() {
            let gateway = GatewayData {
                id: Uuid::new_v4(),
                org_id: Uuid::new_v4(),
                name: "Test Gateway".to_string(),
                slug: "test-gateway".to_string(),
                enabled: true,
            };
            
            let repo = MockGatewayRepository::with_gateway(gateway.clone());
            let result = repo.find_by_slug(gateway.org_id, "test-gateway").await.unwrap();
            
            assert!(result.is_some());
            assert_eq!(result.unwrap().slug, "test-gateway");
        }

        #[tokio::test]
        async fn mock_gateway_returns_servers() {
            let gateway = GatewayData {
                id: Uuid::new_v4(),
                org_id: Uuid::new_v4(),
                name: "Test Gateway".to_string(),
                slug: "test-gateway".to_string(),
                enabled: true,
            };
            let servers = vec![
                GatewayServerData {
                    server_id: Uuid::new_v4(),
                    server_name: "server-1".to_string(),
                    priority: 0,
                },
                GatewayServerData {
                    server_id: Uuid::new_v4(),
                    server_name: "server-2".to_string(),
                    priority: 1,
                },
            ];
            
            let repo = MockGatewayRepository::with_servers(gateway.clone(), servers);
            let result = repo.get_servers(gateway.id).await.unwrap();
            
            assert_eq!(result.len(), 2);
            assert_eq!(result[0].server_name, "server-1");
        }
    }
}
