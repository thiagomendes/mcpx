use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, State},
    http::{request::Parts, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::routes::auth::{extract_token, validate_token};
use crate::AppState;

#[allow(dead_code)]
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let headers = request.headers();

    let token = extract_token(headers)?;
    let _claims = validate_token(&token, &state.config.jwt_secret)?;

    Ok(next.run(request).await)
}

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub org_id: uuid::Uuid,
    pub org_slug: String,
    pub role: String, // owner, admin, member
}

impl AuthUser {
    /// Check if user has a specific permission
    pub fn can(&self, permission: super::permissions::Permission) -> bool {
        super::permissions::has_permission(&self.role, permission)
    }

    /// Require a permission, returning 403 if denied
    pub fn require(
        &self,
        permission: super::permissions::Permission,
    ) -> Result<(), (StatusCode, String)> {
        super::permissions::require_permission(&self.role, permission)
    }
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token(&parts.headers)?;

        // Detect token type and authenticate accordingly
        if token.starts_with("mcpx_pat_") {
            // PAT token - validate via database
            let pat_result = crate::routes::pat::validate_pat(&state.db.pool, &token)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database error: {}", e),
                    )
                })?;

            match pat_result {
                Some(pat) => {
                    // Get user email from database
                    let email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
                        .bind(pat.user_id)
                        .fetch_optional(&state.db.pool)
                        .await
                        .map_err(|e| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Database error: {}", e),
                            )
                        })?
                        .unwrap_or_else(|| String::new());

                    // Get org_slug from database
                    let org_slug: String =
                        sqlx::query_scalar("SELECT slug FROM organizations WHERE id = $1")
                            .bind(pat.org_id)
                            .fetch_optional(&state.db.pool)
                            .await
                            .map_err(|e| {
                                (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    format!("Database error: {}", e),
                                )
                            })?
                            .unwrap_or_else(|| String::new());

                    // Get role from org_members - MUST be a member
                    let role: Option<String> = sqlx::query_scalar(
                        "SELECT role FROM org_members WHERE org_id = $1 AND user_id = $2",
                    )
                    .bind(pat.org_id)
                    .bind(pat.user_id)
                    .fetch_optional(&state.db.pool)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Database error: {}", e),
                        )
                    })?;

                    let role = role.ok_or((
                        StatusCode::UNAUTHORIZED,
                        "User is not a member of this organization".to_string(),
                    ))?;

                    Ok(AuthUser {
                        user_id: pat.user_id,
                        email,
                        org_id: pat.org_id,
                        org_slug,
                        role,
                    })
                }
                None => Err((
                    StatusCode::UNAUTHORIZED,
                    "Invalid or expired PAT".to_string(),
                )),
            }
        } else if token.starts_with("ey") {
            // JWT token - try M2M first, then web session
            use crate::routes::auth::M2MClaims;
            use jsonwebtoken::{decode, DecodingKey, Validation};

            // Try M2M JWT first
            if let Ok(token_data) = decode::<M2MClaims>(
                &token,
                &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
                &Validation::default(),
            ) {
                let claims = token_data.claims;
                if claims.token_type == "m2m" {
                    // It's M2M token - get service account info
                    let sa_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| {
                        (
                            StatusCode::UNAUTHORIZED,
                            "Invalid service account ID".to_string(),
                        )
                    })?;
                    let org_id = uuid::Uuid::parse_str(&claims.org_id)
                        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid org ID".to_string()))?;

                    // Get role from service_accounts table
                    let role: String = sqlx::query_scalar(
                        "SELECT role FROM service_accounts WHERE id = $1 AND org_id = $2",
                    )
                    .bind(sa_id)
                    .bind(org_id)
                    .fetch_optional(&state.db.pool)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Database error: {}", e),
                        )
                    })?
                    .unwrap_or_else(|| "member".to_string());

                    return Ok(AuthUser {
                        user_id: sa_id,                                     // Use SA ID as user_id
                        email: format!("sa-{}@m2m.mcpx", &claims.sub[..8]), // Synthetic email for M2M
                        org_id,
                        org_slug: claims.org_slug,
                        role,
                    });
                }
            }

            // Web session JWT
            let claims = validate_token(&token, &state.config.jwt_secret)?;

            let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap_or_default();
            let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();

            // Fetch role from org_members - MUST be a member
            let role: Option<String> = sqlx::query_scalar(
                "SELECT role FROM org_members WHERE org_id = $1 AND user_id = $2",
            )
            .bind(org_id)
            .bind(user_id)
            .fetch_optional(&state.db.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;

            let role = role.ok_or((
                StatusCode::UNAUTHORIZED,
                "User is not a member of this organization".to_string(),
            ))?;

            Ok(AuthUser {
                user_id,
                email: claims.email,
                org_id,
                org_slug: claims.org_slug,
                role,
            })
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid token format".to_string()))
        }
    }
}
