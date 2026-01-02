//! Organization API Routes
//!
//! Endpoints for org management and members

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::messages::error;
use crate::middleware::auth::AuthUser;
use crate::AppState;

// ============================================
// TYPES
// ============================================

#[derive(Debug, Serialize)]
pub struct OrgMemberResponse {
    pub user_id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct InviteMemberRequest {
    pub email: String,
    pub role: Option<String>,
}

// ============================================
// SQL QUERIES
// ============================================

const SQL_LIST_MEMBERS: &str = r#"
    SELECT 
        om.user_id,
        u.email,
        u.name,
        u.avatar_url,
        om.role,
        om.created_at as joined_at
    FROM org_members om
    JOIN users u ON u.id = om.user_id
    WHERE om.org_id = $1
    ORDER BY om.created_at
"#;

// ============================================
// ROUTE HANDLERS
// ============================================

/// GET /api/orgs/members - List org members
pub async fn list_members(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<OrgMemberResponse>>, (StatusCode, String)> {
    let org_id = auth.org_id;

    let members = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<String>,
            Option<String>,
            String,
            chrono::DateTime<chrono::Utc>,
        ),
    >(SQL_LIST_MEMBERS)
    .bind(org_id)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", error::DATABASE_ERROR, e),
        )
    })?;

    let response: Vec<OrgMemberResponse> = members
        .into_iter()
        .map(
            |(user_id, email, name, avatar_url, role, joined_at)| OrgMemberResponse {
                user_id,
                email,
                name,
                avatar_url,
                role,
                joined_at,
            },
        )
        .collect();

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub message: String,
    pub invite_link: Option<String>,
    pub member: Option<OrgMemberResponse>,
}

/// POST /api/orgs/members/invite - Invite a new member
pub async fn invite_member(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<InviteMemberRequest>,
) -> Result<Json<InviteResponse>, (StatusCode, String)> {
    let user_id = auth.user_id;
    let org_id = auth.org_id;

    // 1. Check permissions (Owner/Admin only)
    if !crate::services::org::OrgService::can_admin(&state.db.pool, org_id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        return Err((
            StatusCode::FORBIDDEN,
            "Only admins can invite members".to_string(),
        ));
    }

    // 2. Check if user exists & is already a member
    let user =
        sqlx::query_as::<_, crate::models::user::User>("SELECT * FROM users WHERE email = $1")
            .bind(&payload.email)
            .fetch_optional(&state.db.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(user) = user {
        // Check if already a member
        let is_member =
            crate::services::org::OrgService::get_user_role(&state.db.pool, org_id, user.id)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("{}: {}", error::DATABASE_ERROR, e),
                    )
                })?
                .is_some();

        if is_member {
            return Err((
                StatusCode::CONFLICT,
                "User is already a member of this organization".to_string(),
            ));
        }
    }

    // 3. Create invite (for both existing and new users)
    let invite = crate::services::org::OrgService::create_invite(
        &state.db.pool,
        org_id,
        &payload.email,
        &payload.role.unwrap_or_else(|| "member".to_string()),
        user_id,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("unique constraint") {
            (
                StatusCode::CONFLICT,
                "Pending invite already exists for this email".to_string(),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create invite: {}", e),
            )
        }
    })?;

    // Construct invite link (using frontend URL from env ideally, for now assuming relative path logic or returning token)
    let invite_link = format!("{}/join/{}", state.config.frontend_url, invite.token);

    Ok(Json(InviteResponse {
        message: "Invite created successfully".to_string(),
        invite_link: Some(invite_link),
        member: None,
    }))
}

/// GET /api/orgs/join/:token - Get invite details
pub async fn get_invite_details(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> Result<Json<crate::models::organization::InviteDetailsResponse>, (StatusCode, String)> {
    let invite =
        crate::services::org::OrgService::get_invite_details_by_token(&state.db.pool, &token)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?
            .ok_or((
                StatusCode::NOT_FOUND,
                "Invalid or expired invite link".to_string(),
            ))?;

    Ok(Json(invite))
}

/// POST /api/orgs/join/:token - Accept an invite
pub async fn join_org(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(token): Path<String>,
) -> Result<Json<OrgMemberResponse>, (StatusCode, String)> {
    let user_id = auth.user_id;
    let user_email = auth.email.to_lowercase();

    // 1. Get invite
    let invite = crate::services::org::OrgService::get_invite_by_token(&state.db.pool, &token)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Invalid or expired invite link".to_string(),
        ))?;

    // 2. SECURITY: Validate that the logged-in user's email matches the invite email
    if invite.email.to_lowercase() != user_email {
        tracing::warn!(
            "Invite email mismatch: invite.email={} user.email={}",
            invite.email,
            user_email
        );
        return Err((
            StatusCode::FORBIDDEN,
            format!(
                "This invite was sent to {}. Please log in with that email address.",
                invite.email
            ),
        ));
    }

    // 3. Add user to org
    let member = crate::services::org::OrgService::add_member(
        &state.db.pool,
        invite.org_id,
        user_id,
        &invite.role,
    )
    .await;

    match member {
        Ok(m) => {
            // 3. Delete invite
            let _ =
                crate::services::org::OrgService::delete_invite(&state.db.pool, invite.id).await;

            // Fetch user details for response
            let user =
                sqlx::query_as::<_, crate::models::user::User>("SELECT * FROM users WHERE id = $1")
                    .bind(user_id)
                    .fetch_one(&state.db.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok(Json(OrgMemberResponse {
                user_id: m.user_id,
                email: user.email,
                name: user.name,
                avatar_url: user.avatar_url,
                role: m.role,
                joined_at: m.created_at,
            }))
        }
        Err(e) => {
            // If unique violation, user already in org. Just delete invite and return success-like or conflict?
            // Let's return Conflict but maybe frontend handles it gracefully by redirecting
            if e.to_string().contains("Duplicate entry")
                || e.to_string().contains("unique constraint")
            {
                let _ = crate::services::org::OrgService::delete_invite(&state.db.pool, invite.id)
                    .await;
                return Err((
                    StatusCode::CONFLICT,
                    "You are already a member of this organization".to_string(),
                ));
            }
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to join organization: {}", e),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct OrgResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_personal: bool,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// POST /api/orgs - Create a new organization
pub async fn create_org(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateOrgRequest>,
) -> Result<Json<OrgResponse>, (StatusCode, String)> {
    let user_id = auth.user_id;

    // Generate slug from name
    let slug = payload
        .name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .replace(" ", "-");

    // Ensure strict slug format
    if slug.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid organization name".to_string(),
        ));
    }

    // Check if slug is taken (simple check, improve later)
    if crate::services::org::OrgService::get_org_by_slug(&state.db.pool, &slug)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .is_some()
    {
        return Err((
            StatusCode::CONFLICT,
            "Organization with this name already exists".to_string(),
        ));
    }

    let org = crate::services::org::OrgService::create_org(
        &state.db.pool,
        &payload.name,
        &slug,
        false, // Not personal
        user_id,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", error::DATABASE_ERROR, e),
        )
    })?;

    Ok(Json(OrgResponse {
        id: org.id,
        name: org.name,
        slug: org.slug,
        is_personal: org.is_personal,
        role: "owner".to_string(),
        created_at: org.created_at,
    }))
}

/// DELETE /api/orgs/:id - Delete an organization (Owner only)
pub async fn delete_org(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(target_org_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = auth.user_id;

    // 1. Check if user is OWNER of the target org
    let role =
        crate::services::org::OrgService::get_user_role(&state.db.pool, target_org_id, user_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("{}: {}", error::DATABASE_ERROR, e),
                )
            })?;

    match role.as_deref() {
        Some("owner") => (), // OK
        Some(_) | None => {
            return Err((
                StatusCode::FORBIDDEN,
                "Only the organization owner can delete it".to_string(),
            ))
        }
    }

    // 2. Check if org is personal (cannot delete personal orgs via this route)
    let org = crate::services::org::OrgService::get_org_by_id(&state.db.pool, target_org_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Organization not found".to_string()))?;

    if org.is_personal {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot delete personal organizations".to_string(),
        ));
    }

    // 3. Delete org
    crate::services::org::OrgService::delete_org(&state.db.pool, target_org_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/orgs/members/:user_id - Remove a member (Owner/Admin only)
pub async fn remove_member(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(target_user_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = auth.user_id;
    let org_id = auth.org_id;

    // 1. Check permissions (Owner/Admin only)
    if !crate::services::org::OrgService::can_admin(&state.db.pool, org_id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        return Err((
            StatusCode::FORBIDDEN,
            "Only admins can remove members".to_string(),
        ));
    }

    // 2. Prevent removing yourself (use leave endpoint if we had one, or just allow it but warn?)
    // Typically "remove member" implies removing SOMEONE ELSE. Leaving is separate.
    // But for simplicity let's allow it, but ensure we don't leave the org ownerless if it's the last owner.
    // For now, simple check:
    if user_id == target_user_id {
        return Err((
            StatusCode::BAD_REQUEST,
            "You cannot remove yourself via this endpoint (use leave)".to_string(),
        ));
    }

    // 3. Remove member
    let removed =
        crate::services::org::OrgService::remove_member(&state.db.pool, org_id, target_user_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("{}: {}", error::DATABASE_ERROR, e),
                )
            })?;

    if !removed {
        return Err((
            StatusCode::NOT_FOUND,
            "Member not found in organization".to_string(),
        ));
    }

    // 4. SECURITY: Delete all PATs the removed user had for this org
    let deleted_pats =
        sqlx::query("DELETE FROM personal_access_tokens WHERE user_id = $1 AND org_id = $2")
            .bind(target_user_id)
            .bind(org_id)
            .execute(&state.db.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("{}: {}", error::DATABASE_ERROR, e),
                )
            })?;

    if deleted_pats.rows_affected() > 0 {
        tracing::info!(
            "Deleted {} PATs for user {} removed from org {}",
            deleted_pats.rows_affected(),
            target_user_id,
            org_id
        );
    }

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/account - Delete user's own account
/// This will:
/// - Delete all organizations where user is the sole owner
/// - Remove user from all other organizations
/// - Delete the user account
pub async fn delete_account(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = auth.user_id;

    // Perform account deletion
    crate::services::org::OrgService::delete_user_account(&state.db.pool, user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    tracing::info!(user_id = %user_id, "User account deleted via API");

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct AccountDeletionPreview {
    pub user_email: String,
    pub orgs_to_delete: Vec<OrgToDelete>,
}

#[derive(Debug, Serialize)]
pub struct OrgToDelete {
    pub id: Uuid,
    pub name: String,
    pub is_personal: bool,
}

/// GET /api/account/deletion-preview - Preview what will be deleted
pub async fn preview_account_deletion(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<AccountDeletionPreview>, (StatusCode, String)> {
    let user_id = auth.user_id;

    // Get user email
    let user = sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    // Get orgs that will be deleted
    let orgs = crate::services::org::OrgService::preview_account_deletion(&state.db.pool, user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    Ok(Json(AccountDeletionPreview {
        user_email: user,
        orgs_to_delete: orgs
            .into_iter()
            .map(|o| OrgToDelete {
                id: o.id,
                name: o.name,
                is_personal: o.is_personal,
            })
            .collect(),
    }))
}
