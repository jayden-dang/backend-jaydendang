use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use async_trait::async_trait;

use crate::domain::{AuthProviderType, OAuthTokenResponse};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scope: Vec<String>,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub state: String,
    pub provider: AuthProviderType,
    pub redirect_url: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub verified_email: Option<bool>,
    pub raw_data: serde_json::Value,
}

#[async_trait]
pub trait OAuthProvider: Send + Sync {
    fn provider_type(&self) -> AuthProviderType;
    fn get_authorization_url(&self, state: &str) -> String;
    async fn exchange_code_for_token(&self, code: &str) -> Result<OAuthTokenResponse>;
    async fn get_user_info(&self, access_token: &str) -> Result<OAuthUserInfo>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokenResponse>;
    fn validate_state(&self, state: &str) -> Result<OAuthState>;
}

pub struct OAuthClient {
    providers: HashMap<AuthProviderType, Box<dyn OAuthProvider>>,
}

impl OAuthClient {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn OAuthProvider>) {
        let provider_type = provider.provider_type();
        self.providers.insert(provider_type, provider);
    }

    pub fn get_provider(&self, provider_type: AuthProviderType) -> Option<&dyn OAuthProvider> {
        self.providers.get(&provider_type).map(|p| p.as_ref())
    }

    pub fn generate_state(provider: AuthProviderType, redirect_url: Option<String>) -> String {
        let oauth_state = OAuthState {
            state: Uuid::new_v4().to_string(),
            provider,
            redirect_url,
            created_at: chrono::Utc::now().timestamp(),
        };

        // In production, you should encrypt this or store it in Redis
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.encode(serde_json::to_string(&oauth_state).unwrap())
    }

    pub fn validate_state(state: &str) -> Result<OAuthState> {
        use base64::{Engine as _, engine::general_purpose};
        let decoded = general_purpose::STANDARD.decode(state)
            .map_err(|_| crate::error::Error::invalid_oauth_state())?;
        
        let oauth_state: OAuthState = serde_json::from_slice(&decoded)
            .map_err(|_| crate::error::Error::invalid_oauth_state())?;

        // Check if state is not older than 10 minutes
        let current_time = chrono::Utc::now().timestamp();
        if current_time - oauth_state.created_at > 600 {
            return Err(crate::error::Error::expired_oauth_state());
        }

        Ok(oauth_state)
    }

    pub async fn handle_oauth_callback(
        &self,
        provider_type: AuthProviderType,
        code: String,
        state: String,
    ) -> Result<(OAuthTokenResponse, OAuthUserInfo, OAuthState)> {
        let provider = self.get_provider(provider_type)
            .ok_or(crate::error::Error::unsupported_oauth_provider())?;

        let oauth_state = Self::validate_state(&state)?;
        let token_response = provider.exchange_code_for_token(&code).await?;
        let user_info = provider.get_user_info(&token_response.access_token).await?;

        Ok((token_response, user_info, oauth_state))
    }
}

impl Default for OAuthClient {
    fn default() -> Self {
        Self::new()
    }
}