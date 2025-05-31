use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::{AuthProviderType, OAuthTokenResponse};
use crate::error::Result;
use super::oauth_client::{OAuthConfig, OAuthProvider, OAuthUserInfo, OAuthState};

#[derive(Debug, Clone)]
pub struct GoogleOAuthProvider {
    config: OAuthConfig,
    http_client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTokenRequest {
    client_id: String,
    client_secret: String,
    code: String,
    grant_type: String,
    redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    expires_in: Option<i64>,
    refresh_token: Option<String>,
    scope: Option<String>,
    token_type: String,
    id_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleUserInfo {
    id: String,
    email: String,
    verified_email: Option<bool>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    locale: Option<String>,
}

impl GoogleOAuthProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let config = OAuthConfig {
            client_id,
            client_secret,
            redirect_uri,
            scope: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            user_info_url: "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
        };

        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl OAuthProvider for GoogleOAuthProvider {
    fn provider_type(&self) -> AuthProviderType {
        AuthProviderType::Google
    }

    fn get_authorization_url(&self, state: &str) -> String {
        let scope_str = self.config.scope.join(" ");
        let response_type_str = "code".to_string();
        let state_str = state.to_string();
        let access_type_str = "offline".to_string();
        let prompt_str = "consent".to_string();
        
        let mut params = HashMap::new();
        params.insert("client_id", &self.config.client_id);
        params.insert("redirect_uri", &self.config.redirect_uri);
        params.insert("scope", &scope_str);
        params.insert("response_type", &response_type_str);
        params.insert("state", &state_str);
        params.insert("access_type", &access_type_str); // For refresh token
        params.insert("prompt", &prompt_str); // Force consent for refresh token

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", self.config.auth_url, query_string)
    }

    async fn exchange_code_for_token(&self, code: &str) -> Result<OAuthTokenResponse> {
        let token_request = GoogleTokenRequest {
            client_id: self.config.client_id.clone(),
            client_secret: self.config.client_secret.clone(),
            code: code.to_string(),
            grant_type: "authorization_code".to_string(),
            redirect_uri: self.config.redirect_uri.clone(),
        };

        let response = self
            .http_client
            .post(&self.config.token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&token_request)
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

        let google_response: GoogleTokenResponse = response
            .json()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        Ok(OAuthTokenResponse {
            access_token: google_response.access_token,
            refresh_token: google_response.refresh_token,
            expires_in: google_response.expires_in,
            token_type: google_response.token_type,
            scope: google_response.scope,
        })
    }

    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo> {
        let response = self
            .http_client
            .get(&self.config.user_info_url)
            .bearer_auth(access_token)
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

        let google_user: GoogleUserInfo = response
            .json()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        let raw_data = serde_json::to_value(&google_user).unwrap();
        
        Ok(OAuthUserInfo {
            id: google_user.id,
            email: google_user.email,
            name: google_user.name,
            picture: google_user.picture,
            verified_email: google_user.verified_email,
            raw_data,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokenResponse> {
        let refresh_token_str = refresh_token.to_string();
        let grant_type_str = "refresh_token".to_string();
        
        let mut params = HashMap::new();
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", &self.config.client_secret);
        params.insert("refresh_token", &refresh_token_str);
        params.insert("grant_type", &grant_type_str);

        let response = self
            .http_client
            .post(&self.config.token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::Error::oauth_error(format!(
                "Token refresh failed: {}",
                error_text
            )));
        }

        let google_response: GoogleTokenResponse = response
            .json()
            .await
            .map_err(|e| crate::error::Error::oauth_error(e.to_string()))?;

        Ok(OAuthTokenResponse {
            access_token: google_response.access_token,
            refresh_token: google_response.refresh_token,
            expires_in: google_response.expires_in,
            token_type: google_response.token_type,
            scope: google_response.scope,
        })
    }

    fn validate_state(&self, state: &str) -> Result<OAuthState> {
        super::oauth_client::OAuthClient::validate_state(state)
    }
}