use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct GitHubOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

#[derive(Debug, Clone)]
pub struct GitHubOAuthClient {
    config: GitHubOAuthConfig,
    client: Client,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub email: Option<String>,
    pub avatar_url: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Error, Debug)]
pub enum OAuthError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("GitHub error: {0} - {1}")]
    GitHub(String, String),
    #[error("Empty access token")]
    EmptyToken,
}

impl GitHubOAuthClient {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            config: GitHubOAuthConfig {
                client_id,
                client_secret,
                redirect_url,
            },
            client,
        }
    }

    pub fn get_auth_url(&self, state: &str) -> String {
        format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user+user:email&state={}",
            self.config.client_id, self.config.redirect_url, state
        )
    }

    pub async fn exchange_code(&self, code: &str) -> Result<String, OAuthError> {
        let params = [
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("code", code),
            ("redirect_uri", self.config.redirect_url.as_str()),
        ];

        let resp = self
            .client
            .post("https://github.com/login/oauth/access_token")
            .form(&params)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| OAuthError::Http(e.to_string()))?;

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| OAuthError::Http(e.to_string()))?;

        if let Some(error) = token_resp.error {
            return Err(OAuthError::GitHub(
                error,
                token_resp.error_description.unwrap_or_default(),
            ));
        }

        token_resp.access_token.ok_or(OAuthError::EmptyToken)
    }

    pub async fn get_user(&self, access_token: &str) -> Result<GitHubUser, OAuthError> {
        let resp = self
            .client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .header("User-Agent", "server-access-manager")
            .send()
            .await
            .map_err(|e| OAuthError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(OAuthError::Http(body));
        }

        resp.json()
            .await
            .map_err(|e| OAuthError::Http(e.to_string()))
    }
}
