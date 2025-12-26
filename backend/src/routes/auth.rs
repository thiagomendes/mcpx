#![allow(dead_code)]
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse, reqwest::async_http_client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::models::user::{User, UserResponse};
use crate::models::organization::{OrgResponse, IdentityResponse};
use crate::services::oauth_provider::{OAuthProvider, OAuthProviderConfig, ProviderUserInfo};
use crate::services::org::{AuthService, OrgService, IdentityService};
use crate::messages::{error, oauth};

// ============================================
// JWT CLAIMS (now includes org context)
// ============================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // user_id
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

fn get_provider_config(state: &AppState, provider: &OAuthProvider) -> Result<OAuthProviderConfig, (StatusCode, String)> {
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
                return Err((StatusCode::NOT_IMPLEMENTED, oauth::MICROSOFT_NOT_CONFIGURED.to_string()));
            }
            Ok(OAuthProviderConfig {
                provider: OAuthProvider::Microsoft,
                client_id,
                client_secret,
                enabled: true,
            })
        },
        OAuthProvider::GitHub => {
            // GitHub config from env (to be added)
            let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
            let client_secret = std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();
            if client_id.is_empty() {
                return Err((StatusCode::NOT_IMPLEMENTED, oauth::GITHUB_NOT_CONFIGURED.to_string()));
            }
            Ok(OAuthProviderConfig {
                provider: OAuthProvider::GitHub,
                client_id,
                client_secret,
                enabled: true,
            })
        },
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
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("{}: {}", error::TOKEN_EXCHANGE_FAILED, e)))?;

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
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    // Create JWT with org context
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
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
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::JWT_ERROR, e)))?;

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
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?
        .ok_or((StatusCode::NOT_FOUND, error::USER_NOT_FOUND.to_string()))?;

    // Get user's orgs
    let orgs = OrgService::list_user_orgs(&state.db.pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    // Get linked identities
    let identities = IdentityService::list_user_identities(&state.db.pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?
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
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?
        .ok_or((StatusCode::FORBIDDEN, "Not a member of this organization".to_string()))?;

    // Get org details
    let org = OrgService::get_org_by_id(&state.db.pool, org_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?
        .ok_or((StatusCode::NOT_FOUND, "Organization not found".to_string()))?;

    // Create new JWT with new org context
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
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
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::JWT_ERROR, e)))?;

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
        .ok_or((StatusCode::UNAUTHORIZED, error::MISSING_AUTH_HEADER.to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, error::INVALID_AUTH_FORMAT.to_string()))?;

    Ok(token.to_string())
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, (StatusCode, String)> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| (StatusCode::UNAUTHORIZED, format!("{}: {}", error::INVALID_TOKEN, e)))
}

pub fn is_dev_mode() -> bool {
    std::env::var("DEV_MODE").map(|v| v == "true" || v == "1").unwrap_or(false)
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

/// Extract org context from JWT claims
pub fn extract_org_context(headers: &axum::http::HeaderMap, secret: &str) -> Result<(Uuid, Uuid, String), (StatusCode, String)> {
    let token = extract_token(headers)?;
    let claims = validate_token(&token, secret)?;
    
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;
    
    Ok((user_id, org_id, claims.org_slug))
}
