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
use crate::services::oauth_provider::{OAuthProvider, OAuthProviderConfig, ProviderUserInfo};
use crate::messages::{error, oauth};

const SQL_UPSERT_USER: &str = r#"
    INSERT INTO users (email, name, google_id, avatar_url, last_login_at)
    VALUES ($1, $2, $3, $4, NOW())
    ON CONFLICT (google_id) DO UPDATE SET
        name = EXCLUDED.name,
        avatar_url = EXCLUDED.avatar_url,
        last_login_at = NOW(),
        updated_at = NOW()
    RETURNING *
"#;

const SQL_SELECT_USER_BY_ID: &str = "SELECT * FROM users WHERE id = $1";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
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
}

fn get_provider_config(state: &AppState, provider: &OAuthProvider) -> Result<OAuthProviderConfig, (StatusCode, String)> {
    match provider {
        OAuthProvider::Google => Ok(OAuthProviderConfig {
            provider: OAuthProvider::Google,
            client_id: state.config.google_client_id.clone(),
            client_secret: state.config.google_client_secret.clone(),
            enabled: !state.config.google_client_id.is_empty(),
        }),
        OAuthProvider::Microsoft => Err((StatusCode::NOT_IMPLEMENTED, oauth::MICROSOFT_NOT_CONFIGURED.to_string())),
        OAuthProvider::GitHub => Err((StatusCode::NOT_IMPLEMENTED, oauth::GITHUB_NOT_CONFIGURED.to_string())),
    }
}

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

    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("{}: {}", error::TOKEN_EXCHANGE_FAILED, e)))?;

    let access_token = token_result.access_token().secret();
    let user_info = ProviderUserInfo::fetch(&provider, access_token)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let user = sqlx::query_as::<_, User>(SQL_UPSERT_USER)
        .bind(&user_info.email)
        .bind(&user_info.name)
        .bind(&user_info.id)
        .bind(&user_info.avatar_url)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?;

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
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

pub async fn google_login(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    oauth_login(State(state), Path("google".to_string())).await
}

pub async fn google_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthCallback>,
) -> Result<Response, (StatusCode, String)> {
    oauth_callback(State(state), Path("google".to_string()), Query(query)).await
}

pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let token = extract_token(&headers)?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, error::INVALID_TOKEN.to_string()))?;

    let user = sqlx::query_as::<_, User>(SQL_SELECT_USER_BY_ID)
        .bind(user_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{}: {}", error::DATABASE_ERROR, e)))?
        .ok_or((StatusCode::NOT_FOUND, error::USER_NOT_FOUND.to_string()))?;

    Ok(Json(UserResponse::from(user)))
}

pub async fn logout() -> impl IntoResponse {
    StatusCode::OK
}

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
