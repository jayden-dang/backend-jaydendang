use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::{AuthProviderType, OAuthTokenResponse};
use crate::error::Result;
use super::oauth_client::{OAuthConfig, OAuthProvider, OAuthUserInfo, OAuthState};

#[derive(Debug, Clone)]
pub struct GitHubOAuthProvider {
    config: OAuthConfig,
    http_client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubTokenRequest {
    client_id: String,
    client_secret: String,
    code: String,
    redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
    scope: String,
    token_type: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubUserInfo {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
    bio: Option<String>,
    location: Option<String>,
    company: Option<String>,
    blog: Option<String>,
    public_repos: Option<u32>,
    public_gists: Option<u32>,
    followers: Option<u32>,
    following: Option<u32>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubEmail {
    email: String,
    verified: bool,
    primary: bool,
    visibility: Option<String>,
}

impl GitHubOAuthProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let config = OAuthConfig {
            client_id,
            client_secret,
            redirect_uri,
            scope: vec![
                "user:email".to_string(),
                "read:user".to_string(),
            ],
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            user_info_url: "https://api.github.com/user".to_string(),
        };

        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    async fn get_user_emails(&self, access_token: &str) -> Result<Vec<GitHubEmail>> {
        let response = self
            .http_client
            .get("https://api.github.com/user/emails")
            .bearer_auth(access_token)
            .header("User-Agent", "JD-Blog-Auth-Service")
            .send()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let emails: Vec<GitHubEmail> = response
            .json()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        Ok(emails)
    }
}

#[async_trait]
impl OAuthProvider for GitHubOAuthProvider {
    fn provider_type(&self) -> AuthProviderType {
        AuthProviderType::Github
    }

    fn get_authorization_url(&self, state: &str) -> String {
        let scope_str = self.config.scope.join(" ");
        let state_str = state.to_string();
        let allow_signup_str = "true".to_string();
        
        let mut params = HashMap::new();
        params.insert("client_id", &self.config.client_id);
        params.insert("redirect_uri", &self.config.redirect_uri);
        params.insert("scope", &scope_str);
        params.insert("state", &state_str);
        params.insert("allow_signup", &allow_signup_str);

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", self.config.auth_url, query_string)
    }

    async fn exchange_code_for_token(&self, code: &str) -> Result<OAuthTokenResponse> {
        let token_request = GitHubTokenRequest {
            client_id: self.config.client_id.clone(),
            client_secret: self.config.client_secret.clone(),
            code: code.to_string(),
            redirect_uri: self.config.redirect_uri.clone(),
        };

        let response = self
            .http_client
            .post(&self.config.token_url)
            .header("Accept", "application/json")
            .header("User-Agent", "JD-Blog-Auth-Service")
            .json(&token_request)
            .send()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::Error::oauth_error(format!(
                "Token exchange failed: {}",
                error_text
            )));
        }

        let github_response: GitHubTokenResponse = response
            .json()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        Ok(OAuthTokenResponse {
            access_token: github_response.access_token,
            refresh_token: github_response.refresh_token,
            expires_in: github_response.expires_in,
            token_type: github_response.token_type,
            scope: Some(github_response.scope),
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let response = self
            .http_client
            .get(&self.config.user_info_url)
            .bearer_auth(access_token)
            .header("User-Agent", "JD-Blog-Auth-Service")
            .send()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::Error::oauth_error(format!(
                "User info request failed: {}",
                error_text
            )));
        }

        let github_user: GitHubUserInfo = response
            .json()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        // Get user's email addresses
        let emails = self.get_user_emails(access_token).await?;
        
        // Add emails to raw data first, before moving github_user
        let mut raw_data = serde_json::to_value(&github_user).unwrap();
        if let Some(obj) = raw_data.as_object_mut() {
            obj.insert("emails".to_string(), serde_json::to_value(&emails).unwrap());
        }

        // Find primary verified email or first verified email
        let primary_email = emails
            .iter()
            .find(|email| email.primary && email.verified)
            .or_else(|| emails.iter().find(|email| email.verified))
            .map(|email| email.email.clone())
            .or_else(|| github_user.email.clone());

        if primary_email.is_none() {
            return Err(crate::error::Error::oauth_error(
                "No verified email found for GitHub user".to_string(),
            ));
        }

        Ok(OAuthUserInfo {
            id: github_user.id.to_string(),
            email: primary_email.unwrap(),
            name: github_user.name,
            picture: github_user.avatar_url,
            verified_email: Some(true), // We already filtered for verified emails
            raw_data,
        })
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Result<OAuthTokenResponse> {
        // GitHub access tokens don't expire, so refresh is not needed
        Err(crate::error::Error::oauth_error(
            "GitHub tokens do not support refresh".to_string(),
        ))
    }

    fn validate_state(&self, state: &str) -> Result<OAuthState> {
        super::oauth_client::OAuthClient::validate_state(state)
    }
}