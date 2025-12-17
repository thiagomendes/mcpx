use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::AppState;
use crate::routes::auth::{extract_token, validate_token, Claims};

/// Middleware to require authentication
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

/// Extension to add user claims to request
#[derive(Clone)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub email: String,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: uuid::Uuid::parse_str(&claims.sub).unwrap_or_default(),
            email: claims.email,
        }
    }
}
