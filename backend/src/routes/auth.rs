#![allow(dead_code)]
use axum::{
    extract::{Form, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use oauth2::{reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::messages::{error, oauth};
use crate::models::organization::{IdentityResponse, OrgResponse};
use crate::models::user::{User, UserResponse};
use crate::services::oauth_provider::{OAuthProvider, OAuthProviderConfig, ProviderUserInfo};
use crate::services::org::{AuthService, IdentityService, OrgService};
use crate::AppState;

// ============================================
// JWT CLAIMS (now includes org context)
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub org_id: String,   // current org context
    pub org_slug: String, // for URL building
    pub provider: String, // current login provider
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct AuthCallback {
    code: String,
    #[allow(dead_code)]
    state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    token: String,
    user: UserResponse,
    orgs: Vec<OrgResponse>,
    current_org: OrgResponse,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    user: UserResponse,
    orgs: Vec<OrgResponse>,
    current_org_id: Uuid,
    identities: Vec<IdentityResponse>,
    auth_provider: String,
}

// ============================================
// PROVIDER CONFIG
// ============================================

fn get_provider_config(
    state: &AppState,
    provider: &OAuthProvider,
) -> Result<OAuthProviderConfig, (StatusCode, String)> {
    match provider {
        OAuthProvider::Google => Ok(OAuthProviderConfig {
            provider: OAuthProvider::Google,
            client_id: state.config.google_client_id.clone(),
            client_secret: state.config.google_client_secret.clone(),
            enabled: !state.config.google_client_id.is_empty(),
        }),
        OAuthProvider::Microsoft => {
            // Microsoft config from env (to be added)
            let client_id = std::env::var("MICROSOFT_CLIENT_ID").unwrap_or_default();
            let client_secret = std::env::var("MICROSOFT_CLIENT_SECRET").unwrap_or_default();
            if client_id.is_empty() {
                return Err((
                    StatusCode::NOT_IMPLEMENTED,
                    oauth::MICROSOFT_NOT_CONFIGURED.to_string(),
                ));
            }
            Ok(OAuthProviderConfig {
                provider: OAuthProvider::Microsoft,
                client_id,
                client_secret,
                enabled: true,
            })
        }
        OAuthProvider::GitHub => {
            // GitHub config from env (to be added)
            let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
            let client_secret = std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();
            if client_id.is_empty() {
                return Err((
                    StatusCode::NOT_IMPLEMENTED,
                    oauth::GITHUB_NOT_CONFIGURED.to_string(),
                ));
            }
            Ok(OAuthProviderConfig {
                provider: OAuthProvider::GitHub,
                client_id,
                client_secret,
                enabled: true,
            })
        }
    }
}

// ============================================
// OAUTH LOGIN FLOW
// ============================================

pub async fn oauth_login(
    State(state): State<Arc<AppState>>,
    Path(provider_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let provider = match provider_name.as_str() {
        "google" => OAuthProvider::Google,
        "microsoft" => OAuthProvider::Microsoft,
        "github" => OAuthProvider::GitHub,
        _ => return Err((StatusCode::BAD_REQUEST, error::UNKNOWN_PROVIDER.to_string())),
    };

    let config = get_provider_config(&state, &provider)?;
    let client = config.create_client(&state.config.base_url);

    let mut auth_request = client.authorize_url(CsrfToken::new_random);
    for scope in provider.scopes() {
        auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
    }

    let (auth_url, _csrf_token) = auth_request.url();
    Ok(Redirect::temporary(auth_url.as_str()))
}

pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Path(provider_name): Path<String>,
    Query(query): Query<AuthCallback>,
) -> Result<Response, (StatusCode, String)> {
    let provider = match provider_name.as_str() {
        "google" => OAuthProvider::Google,
        "microsoft" => OAuthProvider::Microsoft,
        "github" => OAuthProvider::GitHub,
        _ => return Err((StatusCode::BAD_REQUEST, error::UNKNOWN_PROVIDER.to_string())),
    };

    let config = get_provider_config(&state, &provider)?;
    let client = config.create_client(&state.config.base_url);

    // Exchange code for token
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("{}: {}", error::TOKEN_EXCHANGE_FAILED, e),
            )
        })?;

    let access_token = token_result.access_token().secret();

    // Get user info from provider
    let user_info = ProviderUserInfo::fetch(&provider, access_token)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Use AuthService to handle login (find/create user, link identity, create personal org)
    let (user, org) = AuthService::handle_oauth_login(
        &state.db.pool,
        &provider_name,
        &user_info.id,
        &user_info.email,
        user_info.name.as_deref(),
        user_info.avatar_url.as_deref(),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", error::DATABASE_ERROR, e),
        )
    })?;

    // Create JWT with org context
    let jwt_expiry_days =
        crate::routes::settings::get_org_setting_value(&state.db, org.id, "jwt_expiry_days", "30")
            .await
            .parse::<i64>()
            .unwrap_or(30);

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(jwt_expiry_days))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        org_id: org.id.to_string(),
        org_slug: org.slug.clone(),
        provider: provider_name.clone(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", error::JWT_ERROR, e),
        )
    })?;

    let redirect_url = format!("{}?token={}", state.config.frontend_url, token);
    Ok(Redirect::temporary(&redirect_url).into_response())
}

// ============================================
// LEGACY GOOGLE ENDPOINTS (redirect to generic)
// ============================================

pub async fn google_login(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    oauth_login(State(state), Path("google".to_string())).await
}

pub async fn google_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthCallback>,
) -> Result<Response, (StatusCode, String)> {
    oauth_callback(State(state), Path("google".to_string()), Query(query)).await
}

// ============================================
// USER & ORG ENDPOINTS
// ============================================

pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<MeResponse>, (StatusCode, String)> {
    let token = extract_token(&headers)?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;

    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;

    // Get user
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, error::USER_NOT_FOUND.to_string()))?;

    // Get user's orgs
    let orgs = OrgService::list_user_orgs(&state.db.pool, user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?;

    // Get linked identities
    let identities = IdentityService::list_user_identities(&state.db.pool, user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?
        .into_iter()
        .map(IdentityResponse::from)
        .collect();

    Ok(Json(MeResponse {
        user: UserResponse::from(user),
        orgs,
        current_org_id: org_id,
        identities,
        auth_provider: claims.provider,
    }))
}

/// Switch to a different organization - returns new JWT
pub async fn switch_org(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(org_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let token = extract_token(&headers)?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;

    // Verify user is member of target org
    let role = OrgService::get_user_role(&state.db.pool, org_id, user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?
        .ok_or((
            StatusCode::FORBIDDEN,
            "Not a member of this organization".to_string(),
        ))?;

    // Get org details
    let org = OrgService::get_org_by_id(&state.db.pool, org_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("{}: {}", error::DATABASE_ERROR, e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Organization not found".to_string()))?;

    // Create new JWT with new org context
    let jwt_expiry_days =
        crate::routes::settings::get_org_setting_value(&state.db, org.id, "jwt_expiry_days", "30")
            .await
            .parse::<i64>()
            .unwrap_or(30);

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(jwt_expiry_days))
        .unwrap()
        .timestamp() as usize;

    let new_claims = Claims {
        sub: claims.sub,
        email: claims.email,
        org_id: org.id.to_string(),
        org_slug: org.slug.clone(),
        provider: claims.provider.clone(),
        exp: expiration,
    };

    let new_token = encode(
        &Header::default(),
        &new_claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{}: {}", error::JWT_ERROR, e),
        )
    })?;

    Ok(Json(serde_json::json!({
        "token": new_token,
        "org": {
            "id": org.id,
            "name": org.name,
            "slug": org.slug,
            "is_personal": org.is_personal,
            "role": role
        }
    })))
}

pub async fn logout() -> impl IntoResponse {
    StatusCode::OK
}

// ============================================
// HELPERS
// ============================================

pub fn extract_token(headers: &axum::http::HeaderMap) -> Result<String, (StatusCode, String)> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            error::MISSING_AUTH_HEADER.to_string(),
        ))?;

    let token = auth_header.strip_prefix("Bearer ").ok_or((
        StatusCode::UNAUTHORIZED,
        error::INVALID_AUTH_FORMAT.to_string(),
    ))?;

    Ok(token.to_string())
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, (StatusCode, String)> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            format!("{}: {}", error::INVALID_TOKEN, e),
        )
    })
}

pub fn is_dev_mode() -> bool {
    std::env::var("DEV_MODE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

pub fn get_dev_user_id() -> Option<Uuid> {
    if is_dev_mode() {
        std::env::var("DEV_USER_ID")
            .ok()
            .and_then(|id| Uuid::parse_str(&id).ok())
    } else {
        None
    }
}

/// Extract org context from JWT claims (web session only - for backward compatibility)
pub fn extract_org_context(
    headers: &axum::http::HeaderMap,
    secret: &str,
) -> Result<(Uuid, Uuid, String), (StatusCode, String)> {
    let token = extract_token(headers)?;

    // For backward compatibility, try JWT first
    if token.starts_with("ey") {
        let claims = validate_token(&token, secret)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;
        let org_id = Uuid::parse_str(&claims.org_id)
            .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;

        return Ok((user_id, org_id, claims.org_slug));
    }

    // PAT and M2M need async DB access - use extract_org_context_async instead
    Err((
        StatusCode::UNAUTHORIZED,
        "Use extract_org_context_async for PAT/M2M tokens".to_string(),
    ))
}

/// Extract org context supporting PAT, M2M, and web JWT tokens
pub async fn extract_org_context_async(
    headers: &axum::http::HeaderMap,
    secret: &str,
    pool: &sqlx::PgPool,
) -> Result<(Uuid, Uuid, String, String), (StatusCode, String)> {
    let token = extract_token(headers)?;

    // PAT token
    if token.starts_with("mcpx_pat_") {
        let pat = crate::routes::pat::validate_pat(pool, &token)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Invalid or expired PAT".to_string(),
            ))?;

        // Get org_slug
        let org_slug: String = sqlx::query_scalar("SELECT slug FROM organizations WHERE id = $1")
            .bind(pat.org_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?
            .unwrap_or_default();

        // Get role from org_members
        let role: String =
            sqlx::query_scalar("SELECT role FROM org_members WHERE org_id = $1 AND user_id = $2")
                .bind(pat.org_id)
                .bind(pat.user_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database error: {}", e),
                    )
                })?
                .unwrap_or_else(|| "member".to_string());

        return Ok((pat.user_id, pat.org_id, org_slug, role));
    }

    // JWT token (M2M or web session)
    if token.starts_with("ey") {
        use jsonwebtoken::{decode, DecodingKey, Validation};

        // Try M2M first
        if let Ok(token_data) = decode::<M2MClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        ) {
            let claims = token_data.claims;
            if claims.token_type == "m2m" {
                let sa_id = Uuid::parse_str(&claims.sub).map_err(|_| {
                    (
                        StatusCode::UNAUTHORIZED,
                        "Invalid service account ID".to_string(),
                    )
                })?;
                let org_id = Uuid::parse_str(&claims.org_id)
                    .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid org ID".to_string()))?;

                // Get role from service_accounts
                let role: String = sqlx::query_scalar(
                    "SELECT role FROM service_accounts WHERE id = $1 AND org_id = $2",
                )
                .bind(sa_id)
                .bind(org_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database error: {}", e),
                    )
                })?
                .unwrap_or_else(|| "member".to_string());

                return Ok((sa_id, org_id, claims.org_slug, role));
            }
        }

        // Web session JWT
        let claims = validate_token(&token, secret)?;

        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;
        let org_id = Uuid::parse_str(&claims.org_id)
            .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;

        // Get role from org_members
        let role: String =
            sqlx::query_scalar("SELECT role FROM org_members WHERE org_id = $1 AND user_id = $2")
                .bind(org_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database error: {}", e),
                    )
                })?
                .unwrap_or_else(|| "member".to_string());

        return Ok((user_id, org_id, claims.org_slug, role));
    }

    Err((StatusCode::UNAUTHORIZED, "Invalid token format".to_string()))
}

// ============================================
// OAUTH CLIENT CREDENTIALS (M2M)
// ============================================

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Serialize)]
pub struct M2MTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// M2M JWT Claims (different from user JWT)
#[derive(Debug, Serialize, Deserialize)]
pub struct M2MClaims {
    pub sub: String,         // service_account_id
    pub org_id: String,      // org UUID
    pub org_slug: String,    // org slug for proxy URLs
    pub scopes: Vec<String>, // allowed scopes
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub token_type: String, // "m2m" to distinguish from user tokens
}

const M2M_TOKEN_EXPIRY_SECONDS: i64 = 3600; // 1 hour

/// POST /api/auth/token - OAuth 2.0 Client Credentials flow
pub async fn oauth_token(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<TokenRequest>,
) -> Result<Json<M2MTokenResponse>, (StatusCode, String)> {
    // Validate grant_type
    if payload.grant_type != "client_credentials" {
        return Err((
            StatusCode::BAD_REQUEST,
            "unsupported_grant_type".to_string(),
        ));
    }

    // Validate client_id prefix
    if !payload.client_id.starts_with("mcpx_sa_") {
        return Err((StatusCode::UNAUTHORIZED, "invalid_client".to_string()));
    }

    // Validate credentials
    let account = crate::routes::service_accounts::validate_service_account(
        &state.db.pool,
        &payload.client_id,
        &payload.client_secret,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?
    .ok_or((StatusCode::UNAUTHORIZED, "invalid_client".to_string()))?;

    // Get org slug for the JWT
    let org_slug: Option<String> =
        sqlx::query_scalar("SELECT slug FROM organizations WHERE id = $1")
            .bind(account.org_id)
            .fetch_optional(&state.db.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;

    let org_slug = org_slug.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Org not found".to_string(),
    ))?;

    // Extract scopes from JSON
    let scopes: Vec<String> = serde_json::from_value(account.scopes.clone()).unwrap_or_default();

    // Get configurable M2M token expiry (in hours)
    let m2m_expiry_hours = crate::routes::settings::get_org_setting_value(
        &state.db,
        account.org_id,
        "m2m_token_expiry_hours",
        "1",
    )
    .await
    .parse::<i64>()
    .unwrap_or(1);
    let m2m_expiry_seconds = m2m_expiry_hours * 3600;

    // Create M2M JWT
    let now = chrono::Utc::now().timestamp();
    let claims = M2MClaims {
        sub: account.id.to_string(),
        org_id: account.org_id.to_string(),
        org_slug,
        scopes,
        exp: now + m2m_expiry_seconds,
        iat: now,
        iss: "mcpx".to_string(),
        token_type: "m2m".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Token encoding error: {}", e),
        )
    })?;

    tracing::info!(
        "M2M token issued for service account: {}",
        payload.client_id
    );

    Ok(Json(M2MTokenResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: m2m_expiry_seconds,
    }))
}

// ============================================
// UNIT TESTS
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    const TEST_JWT_SECRET: &str = "test-secret-key-for-jwt-testing-32b";

    fn create_test_token(claims: &Claims, secret: &str) -> String {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn create_test_claims() -> Claims {
        Claims {
            sub: Uuid::new_v4().to_string(),
            email: "test@example.com".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            org_id: Uuid::new_v4().to_string(),
            org_slug: "test-org".to_string(),
            provider: "google".to_string(),
        }
    }

    mod extract_token_tests {
        use super::*;

        #[test]
        fn extracts_valid_bearer_token() {
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                header::AUTHORIZATION,
                HeaderValue::from_static("Bearer my-test-token"),
            );

            let result = extract_token(&headers);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "my-test-token");
        }

        #[test]
        fn returns_error_for_missing_header() {
            let headers = axum::http::HeaderMap::new();
            let result = extract_token(&headers);

            assert!(result.is_err());
            let (status, _) = result.unwrap_err();
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }

        #[test]
        fn returns_error_for_invalid_format() {
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                header::AUTHORIZATION,
                HeaderValue::from_static("Basic my-token"),
            );

            let result = extract_token(&headers);
            assert!(result.is_err());
            let (status, _) = result.unwrap_err();
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }

        #[test]
        fn returns_error_for_empty_bearer() {
            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                header::AUTHORIZATION,
                HeaderValue::from_static("Token my-token"),
            );

            let result = extract_token(&headers);
            assert!(result.is_err());
        }
    }

    mod validate_token_tests {
        use super::*;

        #[test]
        fn validates_correct_token() {
            let claims = create_test_claims();
            let token = create_test_token(&claims, TEST_JWT_SECRET);

            let result = validate_token(&token, TEST_JWT_SECRET);
            assert!(result.is_ok());

            let validated_claims = result.unwrap();
            assert_eq!(validated_claims.sub, claims.sub);
            assert_eq!(validated_claims.org_slug, claims.org_slug);
        }

        #[test]
        fn rejects_invalid_secret() {
            let claims = create_test_claims();
            let token = create_test_token(&claims, TEST_JWT_SECRET);

            let result = validate_token(&token, "wrong-secret-key-32-bytes-here!!");
            assert!(result.is_err());

            let (status, _) = result.unwrap_err();
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }

        #[test]
        fn rejects_expired_token() {
            let mut claims = create_test_claims();
            claims.exp = (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize;
            let token = create_test_token(&claims, TEST_JWT_SECRET);

            let result = validate_token(&token, TEST_JWT_SECRET);
            assert!(result.is_err());
        }

        #[test]
        fn rejects_malformed_token() {
            let result = validate_token("not.a.valid.jwt", TEST_JWT_SECRET);
            assert!(result.is_err());

            let (status, _) = result.unwrap_err();
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }
    }

    mod extract_org_context_tests {
        use super::*;

        #[test]
        fn extracts_user_and_org_ids() {
            let claims = create_test_claims();
            let token = create_test_token(&claims, TEST_JWT_SECRET);

            let mut headers = axum::http::HeaderMap::new();
            headers.insert(
                header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );

            let result = extract_org_context(&headers, TEST_JWT_SECRET);
            assert!(result.is_ok());

            let (user_id, org_id, org_slug) = result.unwrap();
            assert_eq!(user_id.to_string(), claims.sub);
            assert_eq!(org_id.to_string(), claims.org_id);
            assert_eq!(org_slug, claims.org_slug);
        }

        #[test]
        fn returns_error_for_missing_auth() {
            let headers = axum::http::HeaderMap::new();
            let result = extract_org_context(&headers, TEST_JWT_SECRET);
            assert!(result.is_err());
        }
    }

    mod m2m_claims_tests {
        use super::*;

        #[test]
        fn m2m_claims_serializes_correctly() {
            let claims = M2MClaims {
                sub: Uuid::new_v4().to_string(),
                org_id: Uuid::new_v4().to_string(),
                org_slug: "test-org".to_string(),
                scopes: vec![
                    "mcp:server:read".to_string(),
                    "mcp:tool:execute".to_string(),
                ],
                token_type: "service_account".to_string(),
                exp: 1234567890,
                iat: 1234564290,
                iss: "mcpx".to_string(),
            };

            let json = serde_json::to_string(&claims).unwrap();
            assert!(json.contains("service_account"));
            assert!(json.contains("mcp:server:read"));
        }

        #[test]
        fn m2m_claims_deserializes_correctly() {
            let json = r#"{
                "sub": "550e8400-e29b-41d4-a716-446655440000",
                "org_id": "550e8400-e29b-41d4-a716-446655440001",
                "org_slug": "test-org",
                "scopes": ["mcp:server:read"],
                "token_type": "service_account",
                "exp": 1234567890,
                "iat": 1234564290,
                "iss": "mcpx"
            }"#;

            let claims: M2MClaims = serde_json::from_str(json).unwrap();
            assert_eq!(claims.token_type, "service_account");
            assert_eq!(claims.scopes.len(), 1);
        }
    }
}
