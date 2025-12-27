#![allow(dead_code)]
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::organization::{Organization, OrgMember, OrgMemberResponse, OrgResponse, UserIdentity};
use crate::models::user::User;

// ============================================
// ORGANIZATION QUERIES
// ============================================

const SQL_CREATE_ORG: &str = r#"
    INSERT INTO organizations (name, slug, is_personal)
    VALUES ($1, $2, $3)
    RETURNING *
"#;

const SQL_GET_ORG_BY_ID: &str = "SELECT * FROM organizations WHERE id = $1";
const SQL_GET_ORG_BY_SLUG: &str = "SELECT * FROM organizations WHERE slug = $1";

const SQL_LIST_USER_ORGS: &str = r#"
    SELECT o.*, om.role
    FROM organizations o
    JOIN org_members om ON o.id = om.org_id
    WHERE om.user_id = $1
    ORDER BY o.is_personal DESC, o.name
"#;

const SQL_UPDATE_ORG: &str = r#"
    UPDATE organizations 
    SET name = COALESCE($2, name), slug = COALESCE($3, slug), updated_at = NOW()
    WHERE id = $1
    RETURNING *
"#;

const SQL_DELETE_ORG: &str = "DELETE FROM organizations WHERE id = $1 AND is_personal = false";

// ============================================
// ORG MEMBER QUERIES
// ============================================

const SQL_ADD_MEMBER: &str = r#"
    INSERT INTO org_members (org_id, user_id, role)
    VALUES ($1, $2, $3)
    RETURNING *
"#;

const SQL_GET_MEMBER: &str = "SELECT * FROM org_members WHERE org_id = $1 AND user_id = $2";

const SQL_LIST_MEMBERS: &str = r#"
    SELECT om.*, u.email, u.name, u.avatar_url
    FROM org_members om
    JOIN users u ON om.user_id = u.id
    WHERE om.org_id = $1
    ORDER BY 
        CASE om.role 
            WHEN 'owner' THEN 1 
            WHEN 'admin' THEN 2 
            ELSE 3 
        END,
        om.created_at
"#;

const SQL_UPDATE_MEMBER_ROLE: &str = r#"
    UPDATE org_members SET role = $3 WHERE org_id = $1 AND user_id = $2
"#;

const SQL_REMOVE_MEMBER: &str = "DELETE FROM org_members WHERE org_id = $1 AND user_id = $2";

const SQL_COUNT_OWNERS: &str = "SELECT COUNT(*) FROM org_members WHERE org_id = $1 AND role = 'owner'";

// ============================================
// INVITE QUERIES
// ============================================

const SQL_CREATE_INVITE: &str = r#"
    INSERT INTO org_invites (org_id, email, role, token, expires_at, created_by)
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING *
"#;

const SQL_GET_INVITE_BY_TOKEN: &str = "SELECT * FROM org_invites WHERE token = $1 AND expires_at > NOW()";
const SQL_DELETE_INVITE: &str = "DELETE FROM org_invites WHERE id = $1";
const SQL_LIST_INVITES: &str = "SELECT * FROM org_invites WHERE org_id = $1 ORDER BY created_at DESC";


// ============================================
// USER IDENTITY QUERIES
// ============================================

const SQL_CREATE_IDENTITY: &str = r#"
    INSERT INTO user_identities (user_id, provider, provider_user_id)
    VALUES ($1, $2, $3)
    ON CONFLICT (provider, provider_user_id) DO UPDATE SET user_id = $1
    RETURNING *
"#;

const SQL_GET_IDENTITY: &str = "SELECT * FROM user_identities WHERE provider = $1 AND provider_user_id = $2";
const SQL_LIST_USER_IDENTITIES: &str = "SELECT * FROM user_identities WHERE user_id = $1";

// ============================================
// USER QUERIES (for auth)
// ============================================

const SQL_CREATE_USER: &str = r#"
    INSERT INTO users (email, name, avatar_url, last_login_at)
    VALUES ($1, $2, $3, NOW())
    RETURNING *
"#;

const SQL_GET_USER_BY_EMAIL: &str = "SELECT * FROM users WHERE email = $1";
const SQL_GET_USER_BY_ID: &str = "SELECT * FROM users WHERE id = $1";

const SQL_UPDATE_USER_LOGIN: &str = r#"
    UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1
"#;

// ============================================
// ORGANIZATION SERVICE
// ============================================

pub struct OrgService;

impl OrgService {
    /// Create a new organization and add the creator as owner
    pub async fn create_org(
        pool: &PgPool,
        name: &str,
        slug: &str,
        is_personal: bool,
        owner_user_id: Uuid,
    ) -> Result<Organization, sqlx::Error> {
        let org = sqlx::query_as::<_, Organization>(SQL_CREATE_ORG)
            .bind(name)
            .bind(slug)
            .bind(is_personal)
            .fetch_one(pool)
            .await?;

        // Add creator as owner
        sqlx::query(SQL_ADD_MEMBER)
            .bind(org.id)
            .bind(owner_user_id)
            .bind("owner")
            .execute(pool)
            .await?;

        Ok(org)
    }

    /// Create a personal workspace for a new user
    pub async fn create_personal_org(pool: &PgPool, user: &User) -> Result<Organization, sqlx::Error> {
        // Generate slug from email prefix
        let slug = user.email
            .split('@')
            .next()
            .unwrap_or("user")
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();

        // Ensure unique slug by appending random suffix if needed
        let unique_slug = format!("{}-{}", slug, &Uuid::new_v4().to_string()[..8]);

        Self::create_org(pool, "Personal", &unique_slug, true, user.id).await
    }

    pub async fn get_org_by_id(pool: &PgPool, org_id: Uuid) -> Result<Option<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(SQL_GET_ORG_BY_ID)
            .bind(org_id)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_org_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(SQL_GET_ORG_BY_SLUG)
            .bind(slug)
            .fetch_optional(pool)
            .await
    }

    /// List all organizations the user is a member of
    pub async fn list_user_orgs(pool: &PgPool, user_id: Uuid) -> Result<Vec<OrgResponse>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct OrgWithRole {
            id: Uuid,
            name: String,
            slug: String,
            is_personal: bool,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            role: String,
        }

        let orgs = sqlx::query_as::<_, OrgWithRole>(SQL_LIST_USER_ORGS)
            .bind(user_id)
            .fetch_all(pool)
            .await?;

        Ok(orgs.into_iter().map(|o| OrgResponse {
            id: o.id,
            name: o.name,
            slug: o.slug,
            is_personal: o.is_personal,
            role: o.role,
            created_at: o.created_at,
        }).collect())
    }

    pub async fn update_org(
        pool: &PgPool,
        org_id: Uuid,
        name: Option<&str>,
        slug: Option<&str>,
    ) -> Result<Organization, sqlx::Error> {
        sqlx::query_as::<_, Organization>(SQL_UPDATE_ORG)
            .bind(org_id)
            .bind(name)
            .bind(slug)
            .fetch_one(pool)
            .await
    }

    pub async fn delete_org(pool: &PgPool, org_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(SQL_DELETE_ORG)
            .bind(org_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Check if user has permission in org
    pub async fn get_user_role(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<Option<String>, sqlx::Error> {
        let member = sqlx::query_as::<_, OrgMember>(SQL_GET_MEMBER)
            .bind(org_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
        Ok(member.map(|m| m.role))
    }

    /// Check if user can perform admin actions (owner or admin role)
    pub async fn can_admin(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let role = Self::get_user_role(pool, org_id, user_id).await?;
        Ok(matches!(role.as_deref(), Some("owner") | Some("admin")))
    }

    // ============================================
    // MEMBER MANAGEMENT
    // ============================================

    pub async fn add_member(
        pool: &PgPool,
        org_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<OrgMember, sqlx::Error> {
        sqlx::query_as::<_, OrgMember>(SQL_ADD_MEMBER)
            .bind(org_id)
            .bind(user_id)
            .bind(role)
            .fetch_one(pool)
            .await
    }

    pub async fn list_members(pool: &PgPool, org_id: Uuid) -> Result<Vec<OrgMemberResponse>, sqlx::Error> {
        #[derive(sqlx::FromRow)]
        struct MemberWithUser {
            id: Uuid,
            org_id: Uuid,
            user_id: Uuid,
            role: String,
            created_at: chrono::DateTime<chrono::Utc>,
            email: String,
            name: Option<String>,
            avatar_url: Option<String>,
        }

        let members = sqlx::query_as::<_, MemberWithUser>(SQL_LIST_MEMBERS)
            .bind(org_id)
            .fetch_all(pool)
            .await?;

        Ok(members.into_iter().map(|m| OrgMemberResponse {
            id: m.id,
            user_id: m.user_id,
            email: m.email,
            name: m.name,
            avatar_url: m.avatar_url,
            role: m.role,
            created_at: m.created_at,
        }).collect())
    }

    pub async fn update_member_role(
        pool: &PgPool,
        org_id: Uuid,
        user_id: Uuid,
        new_role: &str,
    ) -> Result<bool, sqlx::Error> {
        // Prevent removing the last owner
        if new_role != "owner" {
            let count: (i64,) = sqlx::query_as(SQL_COUNT_OWNERS)
                .bind(org_id)
                .fetch_one(pool)
                .await?;
            
            let current_role = Self::get_user_role(pool, org_id, user_id).await?;
            if current_role.as_deref() == Some("owner") && count.0 <= 1 {
                return Ok(false); // Can't demote the last owner
            }
        }

        let result = sqlx::query(SQL_UPDATE_MEMBER_ROLE)
            .bind(org_id)
            .bind(user_id)
            .bind(new_role)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn remove_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        // Prevent removing the last owner
        let current_role = Self::get_user_role(pool, org_id, user_id).await?;
        if current_role.as_deref() == Some("owner") {
            let count: (i64,) = sqlx::query_as(SQL_COUNT_OWNERS)
                .bind(org_id)
                .fetch_one(pool)
                .await?;
            if count.0 <= 1 {
                return Ok(false); // Can't remove the last owner
            }
        }

        let result = sqlx::query(SQL_REMOVE_MEMBER)
            .bind(org_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    // ============================================
    // INVITE MANAGEMENT
    // ============================================

    pub async fn create_invite(
        pool: &PgPool,
        org_id: Uuid,
        email: &str,
        role: &str,
        created_by: Uuid,
    ) -> Result<crate::models::organization::OrgInvite, sqlx::Error> {
        // Generate random token
        let token = Uuid::new_v4().to_string();
        // Expires in 7 days
        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        sqlx::query_as::<_, crate::models::organization::OrgInvite>(SQL_CREATE_INVITE)
            .bind(org_id)
            .bind(email)
            .bind(role)
            .bind(token)
            .bind(expires_at)
            .bind(created_by)
            .fetch_one(pool)
            .await
    }

    pub async fn get_invite_details_by_token(
        pool: &PgPool,
        token: &str,
    ) -> Result<Option<crate::models::organization::InviteDetailsResponse>, sqlx::Error> {
        let sql = r#"
            SELECT 
                i.email,
                i.org_id,
                o.name as org_name,
                i.role,
                u.name as inviter_name,
                u.email as inviter_email
            FROM org_invites i
            JOIN organizations o ON i.org_id = o.id
            LEFT JOIN users u ON i.created_by = u.id
            WHERE i.token = $1 AND i.expires_at > NOW()
        "#;

        sqlx::query_as::<_, crate::models::organization::InviteDetailsResponse>(sql)
            .bind(token)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_invite_by_token(pool: &PgPool, token: &str) -> Result<Option<crate::models::organization::OrgInvite>, sqlx::Error> {
        sqlx::query_as::<_, crate::models::organization::OrgInvite>(SQL_GET_INVITE_BY_TOKEN)
            .bind(token)
            .fetch_optional(pool)
            .await
    }

    pub async fn delete_invite(pool: &PgPool, invite_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(SQL_DELETE_INVITE)
            .bind(invite_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Delete a user account completely
    /// - Deletes orgs where user is the ONLY owner
    /// - Removes user from all other org memberships
    /// - Deletes user's personal org (with CASCADE to all resources)
    /// - Deletes user identities
    /// - Deletes the user
    pub async fn delete_user_account(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        // Start transaction
        let mut tx = pool.begin().await?;

        // 1. Find all orgs where this user is the ONLY owner (these will be deleted)
        let owned_orgs: Vec<Organization> = sqlx::query_as(
            r#"
            SELECT o.* FROM organizations o
            JOIN org_members om ON o.id = om.org_id
            WHERE om.user_id = $1 AND om.role = 'owner'
            AND (SELECT COUNT(*) FROM org_members WHERE org_id = o.id AND role = 'owner') = 1
            "#
        )
            .bind(user_id)
            .fetch_all(&mut *tx)
            .await?;

        // 2. Delete these orgs (CASCADE will handle resources)
        for org in &owned_orgs {
            tracing::info!(org_id = %org.id, org_name = %org.name, "Deleting org owned by deleted user");
            sqlx::query("DELETE FROM organizations WHERE id = $1")
                .bind(org.id)
                .execute(&mut *tx)
                .await?;
        }

        // 3. Delete user from remaining org memberships
        sqlx::query("DELETE FROM org_members WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // 4. Delete user identities
        sqlx::query("DELETE FROM user_identities WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // 5. Delete the user
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        tracing::info!(user_id = %user_id, deleted_orgs = owned_orgs.len(), "User account deleted");
        Ok(())
    }

    /// Preview what will be deleted when deleting a user account
    pub async fn preview_account_deletion(pool: &PgPool, user_id: Uuid) -> Result<Vec<OrgResponse>, sqlx::Error> {
        // Find all orgs where this user is the ONLY owner (these will be deleted)
        #[derive(sqlx::FromRow)]
        struct OrgWithRole {
            id: Uuid,
            name: String,
            slug: String,
            is_personal: bool,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let owned_orgs: Vec<OrgWithRole> = sqlx::query_as(
            r#"
            SELECT o.id, o.name, o.slug, o.is_personal, o.created_at FROM organizations o
            JOIN org_members om ON o.id = om.org_id
            WHERE om.user_id = $1 AND om.role = 'owner'
            AND (SELECT COUNT(*) FROM org_members WHERE org_id = o.id AND role = 'owner') = 1
            "#
        )
            .bind(user_id)
            .fetch_all(pool)
            .await?;

        Ok(owned_orgs.into_iter().map(|o| OrgResponse {
            id: o.id,
            name: o.name,
            slug: o.slug,
            is_personal: o.is_personal,
            role: "owner".to_string(),
            created_at: o.created_at,
        }).collect())
    }
}

// ============================================
// IDENTITY SERVICE (multi-provider auth)
// ============================================

pub struct IdentityService;

impl IdentityService {
    /// Find user by OAuth provider identity
    pub async fn find_by_provider(
        pool: &PgPool,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<UserIdentity>, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(SQL_GET_IDENTITY)
            .bind(provider)
            .bind(provider_user_id)
            .fetch_optional(pool)
            .await
    }

    /// Link a new provider identity to an existing user
    pub async fn create_or_link(
        pool: &PgPool,
        user_id: Uuid,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<UserIdentity, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(SQL_CREATE_IDENTITY)
            .bind(user_id)
            .bind(provider)
            .bind(provider_user_id)
            .fetch_one(pool)
            .await
    }

    /// List all linked identities for a user
    pub async fn list_user_identities(pool: &PgPool, user_id: Uuid) -> Result<Vec<UserIdentity>, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(SQL_LIST_USER_IDENTITIES)
            .bind(user_id)
            .fetch_all(pool)
            .await
    }
}

// ============================================
// AUTH SERVICE (user creation with org)
// ============================================

pub struct AuthService;

impl AuthService {
    /// Handle OAuth callback - find or create user, create personal org if new
    pub async fn handle_oauth_login(
        pool: &PgPool,
        provider: &str,
        provider_user_id: &str,
        email: &str,
        name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<(User, Organization), sqlx::Error> {
        // 1. Check if identity already exists
        if let Some(identity) = IdentityService::find_by_provider(pool, provider, provider_user_id).await? {
            let user = sqlx::query_as::<_, User>(SQL_GET_USER_BY_ID)
                .bind(identity.user_id)
                .fetch_one(pool)
                .await?;
            
            // Update last login
            sqlx::query(SQL_UPDATE_USER_LOGIN)
                .bind(user.id)
                .execute(pool)
                .await?;

            // Get user's personal org (or first org)
            let orgs = OrgService::list_user_orgs(pool, user.id).await?;
            let org = if let Some(personal) = orgs.iter().find(|o| o.is_personal) {
                OrgService::get_org_by_id(pool, personal.id).await?.unwrap()
            } else if let Some(first) = orgs.first() {
                OrgService::get_org_by_id(pool, first.id).await?.unwrap()
            } else {
                // Shouldn't happen, but create personal org if missing
                OrgService::create_personal_org(pool, &user).await?
            };

            return Ok((user, org));
        }

        // 2. Check if user exists by email (account merge)
        if let Some(existing_user) = sqlx::query_as::<_, User>(SQL_GET_USER_BY_EMAIL)
            .bind(email)
            .fetch_optional(pool)
            .await?
        {
            // Link new identity to existing user
            IdentityService::create_or_link(pool, existing_user.id, provider, provider_user_id).await?;
            
            // Update last login
            sqlx::query(SQL_UPDATE_USER_LOGIN)
                .bind(existing_user.id)
                .execute(pool)
                .await?;

            let orgs = OrgService::list_user_orgs(pool, existing_user.id).await?;
            let org = if let Some(personal) = orgs.iter().find(|o| o.is_personal) {
                OrgService::get_org_by_id(pool, personal.id).await?.unwrap()
            } else {
                OrgService::create_personal_org(pool, &existing_user).await?
            };

            return Ok((existing_user, org));
        }

        // 3. Create new user + personal org
        let user = sqlx::query_as::<_, User>(SQL_CREATE_USER)
            .bind(email)
            .bind(name)
            .bind(avatar_url)
            .fetch_one(pool)
            .await?;

        IdentityService::create_or_link(pool, user.id, provider, provider_user_id).await?;
        let org = OrgService::create_personal_org(pool, &user).await?;

        Ok((user, org))
    }
}
