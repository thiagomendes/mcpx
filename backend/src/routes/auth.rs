use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
    basic::BasicClient,
    reqwest::async_http_client,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::models::user::{User, UserResponse};

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub exp: usize,
}

// Google user info response
#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    id: String,
    email: String,
    name: Option<String>,
    picture: Option<String>,
}

// OAuth callback query params
#[derive(Debug, Deserialize)]
pub struct AuthCallback {
    code: String,
    #[allow(dead_code)]
    state: Option<String>,
}

// Auth response with JWT token
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    token: String,
    user: UserResponse,
}

fn create_oauth_client(state: &AppState) -> BasicClient {
    BasicClient::new(
        ClientId::new(state.config.google_client_id.clone()),
        Some(ClientSecret::new(state.config.google_client_secret.clone())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
    )
    .set_redirect_uri(
        RedirectUrl::new(format!("{}/api/auth/google/callback", "http://localhost:8080")).unwrap(),
    )
}

// GET /api/auth/google - Redirect to Google OAuth
pub async fn google_login(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let client = create_oauth_client(&state);
    
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Redirect::temporary(auth_url.as_str())
}

// GET /api/auth/google/callback - Handle OAuth callback
pub async fn google_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuthCallback>,
) -> Result<Response, (StatusCode, String)> {
    let client = create_oauth_client(&state);

    // Exchange code for token
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Token exchange failed: {}", e)))?;

    // Get user info from Google
    let access_token = token_result.access_token().secret();
    let user_info: GoogleUserInfo = reqwest::Client::new()
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to get user info: {}", e)))?
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to parse user info: {}", e)))?;

    // Create or update user in database
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, name, google_id, avatar_url, last_login_at)
        VALUES ($1, $2, $3, $4, NOW())
        ON CONFLICT (google_id) DO UPDATE SET
            name = EXCLUDED.name,
            avatar_url = EXCLUDED.avatar_url,
            last_login_at = NOW(),
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(&user_info.email)
    .bind(&user_info.name)
    .bind(&user_info.id)
    .bind(&user_info.picture)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    // Generate JWT token (30 days expiration)
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
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JWT error: {}", e)))?;

    // Redirect to frontend with token
    let redirect_url = format!("{}?token={}", state.config.frontend_url, token);
    
    Ok(Redirect::temporary(&redirect_url).into_response())
}

// GET /api/auth/me - Get current user
pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserResponse>, (StatusCode, String)> {
    let token = extract_token(&headers)?;
    let claims = validate_token(&token, &state.config.jwt_secret)?;
    
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(UserResponse::from(user)))
}

// POST /api/auth/logout - Logout (client-side token removal)
pub async fn logout() -> impl IntoResponse {
    StatusCode::OK
}

// Helper: Extract token from Authorization header
pub fn extract_token(headers: &axum::http::HeaderMap) -> Result<String, (StatusCode, String)> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid authorization format".to_string()))?;

    Ok(token.to_string())
}

// Helper: Validate JWT token
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, (StatusCode, String)> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))
}
