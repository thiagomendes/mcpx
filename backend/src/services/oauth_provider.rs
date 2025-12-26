#![allow(dead_code)]
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    Microsoft,
    GitHub,
}

impl OAuthProvider {
    pub fn auth_url(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
            OAuthProvider::Microsoft => "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            OAuthProvider::GitHub => "https://github.com/login/oauth/authorize",
        }
    }

    pub fn token_url(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://oauth2.googleapis.com/token",
            OAuthProvider::Microsoft => "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            OAuthProvider::GitHub => "https://github.com/login/oauth/access_token",
        }
    }

    pub fn userinfo_url(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://www.googleapis.com/oauth2/v2/userinfo",
            OAuthProvider::Microsoft => "https://graph.microsoft.com/v1.0/me",
            OAuthProvider::GitHub => "https://api.github.com/user",
        }
    }

    pub fn scopes(&self) -> Vec<&'static str> {
        match self {
            OAuthProvider::Google => vec!["email", "profile"],
            OAuthProvider::Microsoft => vec!["openid", "email", "profile", "User.Read"],
            OAuthProvider::GitHub => vec!["user:email", "read:user"],
        }
    }

    pub fn callback_path(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "/api/auth/google/callback",
            OAuthProvider::Microsoft => "/api/auth/microsoft/callback",
            OAuthProvider::GitHub => "/api/auth/github/callback",
        }
    }
}

#[derive(Debug, Clone)]
pub struct OAuthProviderConfig {
    pub provider: OAuthProvider,
    pub client_id: String,
    pub client_secret: String,
    pub enabled: bool,
}

impl OAuthProviderConfig {
    pub fn create_client(&self, base_url: &str) -> BasicClient {
        let redirect_url = format!("{}{}", base_url, self.provider.callback_path());

        BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(self.provider.auth_url().to_string()).unwrap(),
            Some(TokenUrl::new(self.provider.token_url().to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
    }
}

#[derive(Debug, Deserialize)]
pub struct ProviderUserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

impl ProviderUserInfo {
    pub async fn fetch(provider: &OAuthProvider, access_token: &str) -> Result<Self, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(provider.userinfo_url())
            .bearer_auth(access_token)
            .header("Accept", "application/json")
            .header("User-Agent", "mcpx")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch user info: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user info: {}", e))?;

        match provider {
            OAuthProvider::Google => Ok(Self {
                id: json["id"].as_str().unwrap_or_default().to_string(),
                email: json["email"].as_str().unwrap_or_default().to_string(),
                name: json["name"].as_str().map(String::from),
                avatar_url: json["picture"].as_str().map(String::from),
            }),
            OAuthProvider::Microsoft => Ok(Self {
                id: json["id"].as_str().unwrap_or_default().to_string(),
                email: json["mail"].as_str()
                    .or(json["userPrincipalName"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                name: json["displayName"].as_str().map(String::from),
                avatar_url: None,
            }),
            OAuthProvider::GitHub => Ok(Self {
                id: json["id"].to_string(),
                email: json["email"].as_str().unwrap_or_default().to_string(),
                name: json["name"].as_str().map(String::from),
                avatar_url: json["avatar_url"].as_str().map(String::from),
            }),
        }
    }
}
