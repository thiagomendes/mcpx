use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, State},
    http::{request::Parts, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::AppState;
use crate::routes::auth::{extract_token, validate_token, Claims};

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
#[allow(dead_code)]

#[derive(Clone, Debug)]
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

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
    
        let token = extract_token(&parts.headers)?;
        
    
        let claims = validate_token(&token, &state.config.jwt_secret)?;
        
        Ok(AuthUser::from(claims))
    }
}
