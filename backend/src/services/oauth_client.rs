use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use uuid::Uuid;

// --- Database Types ---

#[derive(Debug, FromRow)]
pub struct ServerOAuthConfig {
    pub id: Uuid,
    pub oauth_authorization_url: Option<String>,
    pub oauth_token_url: Option<String>,
    pub oauth_client_id: Option<String>,
    pub oauth_scopes: Option<String>,
    pub oauth_use_pkce: Option<bool>,
}

#[derive(Debug, FromRow)]
pub struct PendingOAuth {
    pub id: Uuid,
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub pkce_code_verifier: Option<String>,
    pub oauth_token_url: Option<String>,
    pub oauth_client_id: Option<String>,
    pub oauth_use_pkce: Option<bool>,
}

// --- Token Types ---

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: Option<String>,
    expires_in: Option<i64>,
    scope: Option<String>,
}

// --- PKCE ---

/// Generate a cryptographically random code verifier (43-128 chars)
fn generate_code_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Generate code challenge from verifier using S256 method
fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generate a random state parameter
fn generate_state() -> String {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

// --- Authorization URL Generation ---

/// Generate authorization URL with optional PKCE
/// Returns (authorization_url, state, code_verifier)
pub fn generate_authorization_url(
    config: &ServerOAuthConfig,
    callback_url: &str,
) -> (String, String, Option<String>) {
    let authorization_url = config.oauth_authorization_url.as_ref()
        .expect("OAuth authorization URL required");
    let client_id = config.oauth_client_id.as_ref()
        .expect("OAuth client ID required");
    
    let state = generate_state();
    let use_pkce = config.oauth_use_pkce.unwrap_or(true);
    
    let mut url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&state={}",
        authorization_url,
        urlencoding::encode(client_id),
        urlencoding::encode(callback_url),
        urlencoding::encode(&state),
    );
    
    // Add scopes if specified
    if let Some(scopes) = &config.oauth_scopes {
        url.push_str(&format!("&scope={}", urlencoding::encode(scopes)));
    }
    
    // Add PKCE parameters if enabled (OAuth 2.1)
    let code_verifier = if use_pkce {
        let verifier = generate_code_verifier();
        let challenge = generate_code_challenge(&verifier);
        url.push_str(&format!(
            "&code_challenge={}&code_challenge_method=S256",
            urlencoding::encode(&challenge)
        ));
        Some(verifier)
    } else {
        None
    };
    
    (url, state, code_verifier)
}

// --- Token Exchange ---

/// Exchange authorization code for tokens
pub async fn exchange_code(
    pending: &PendingOAuth,
    code: &str,
    callback_url: &str,
    client_secret: Option<&str>,
) -> Result<OAuthTokens, String> {
    let token_url = pending.oauth_token_url.as_ref()
        .ok_or("Token URL not configured")?;
    let client_id = pending.oauth_client_id.as_ref()
        .ok_or("Client ID not configured")?;
    
    let mut params = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code.to_string()),
        ("redirect_uri", callback_url.to_string()),
        ("client_id", client_id.clone()),
    ];
    
    // Add client_secret if available (confidential client)
    if let Some(secret) = client_secret {
        params.push(("client_secret", secret.to_string()));
    }
    
    // Add PKCE code_verifier if used
    if let Some(verifier) = &pending.pkce_code_verifier {
        params.push(("code_verifier", verifier.clone()));
    }
    
    let client = Client::new();
    let response = client
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token request failed: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed: {}", error_text));
    }
    
    let token_response: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;
    
    // Calculate expiration time
    let expires_at = token_response.expires_in
        .map(|secs| Utc::now() + Duration::seconds(secs));
    
    Ok(OAuthTokens {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        token_type: token_response.token_type.unwrap_or_else(|| "Bearer".to_string()),
        expires_at,
        scope: token_response.scope,
    })
}

/// Refresh an expired access token
pub async fn refresh_access_token(
    token_url: &str,
    client_id: &str,
    client_secret: Option<&str>,
    refresh_token: &str,
) -> Result<OAuthTokens, String> {
    let mut params = vec![
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh_token.to_string()),
        ("client_id", client_id.to_string()),
    ];
    
    if let Some(secret) = client_secret {
        params.push(("client_secret", secret.to_string()));
    }
    
    let client = Client::new();
    let response = client
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Refresh request failed: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed: {}", error_text));
    }
    
    let token_response: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse refresh response: {}", e))?;
    
    let expires_at = token_response.expires_in
        .map(|secs| Utc::now() + Duration::seconds(secs));
    
    Ok(OAuthTokens {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        token_type: token_response.token_type.unwrap_or_else(|| "Bearer".to_string()),
        expires_at,
        scope: token_response.scope,
    })
}
