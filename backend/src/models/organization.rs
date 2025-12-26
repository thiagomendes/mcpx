#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================
// ORGANIZATION
// ============================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_personal: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub name: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrgResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_personal: bool,
    pub role: String,  // user's role in this org
    pub created_at: DateTime<Utc>,
}

impl OrgResponse {
    pub fn from_org_with_role(org: Organization, role: &str) -> Self {
        Self {
            id: org.id,
            name: org.name,
            slug: org.slug,
            is_personal: org.is_personal,
            role: role.to_string(),
            created_at: org.created_at,
        }
    }
}

// ============================================
// ORG MEMBER
// ============================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum OrgRole {
    Owner,
    Admin,
    Member,
}

impl std::fmt::Display for OrgRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrgRole::Owner => write!(f, "owner"),
            OrgRole::Admin => write!(f, "admin"),
            OrgRole::Member => write!(f, "member"),
        }
    }
}

impl std::str::FromStr for OrgRole {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(OrgRole::Owner),
            "admin" => Ok(OrgRole::Admin),
            "member" => Ok(OrgRole::Member),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OrgMember {
    pub id: Uuid,
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrgMemberResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: String,
}

// ============================================
// USER IDENTITY (multi-provider auth)
// ============================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserIdentity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentityResponse {
    pub provider: String,
    pub created_at: DateTime<Utc>,
}

impl From<UserIdentity> for IdentityResponse {
    fn from(identity: UserIdentity) -> Self {
        Self {
            provider: identity.provider,
            created_at: identity.created_at,
        }
    }
}

// ============================================
// ORG INVITE
// ============================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct OrgInvite {
    pub id: Uuid,
    pub org_id: Uuid,
    pub email: String,
    pub role: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct InviteDetailsResponse {
    pub email: String,
    pub org_id: Uuid,
    pub org_name: String,
    pub role: String,
    pub inviter_name: Option<String>,
    pub inviter_email: Option<String>,
}
