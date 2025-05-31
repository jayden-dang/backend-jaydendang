use serde::{Deserialize, Serialize};
use std::fmt::Display;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "auth_provider", rename_all = "lowercase")]
pub enum AuthProviderType {
    Email,
    Google,
    Github,
    Wallet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "provider_status", rename_all = "lowercase")]
pub enum ProviderStatus {
    Active,
    Suspended,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAuthProvider {
    pub provider_id: Uuid,
    pub user_id: Uuid,
    pub provider_type: AuthProviderType,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub password_hash: Option<String>,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>,
    pub oauth_access_token: Option<String>,
    pub oauth_refresh_token: Option<String>,
    pub oauth_token_expires_at: Option<OffsetDateTime>,
    pub provider_metadata: Option<serde_json::Value>,
    pub status: ProviderStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_used_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthProviderForCreate {
    pub user_id: Uuid,
    pub provider_type: AuthProviderType,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub password_hash: Option<String>,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>,
    pub oauth_access_token: Option<String>,
    pub oauth_refresh_token: Option<String>,
    pub oauth_token_expires_at: Option<OffsetDateTime>,
    pub provider_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthProviderForUpdate {
    pub provider_email: Option<String>,
    pub password_hash: Option<String>,
    pub oauth_access_token: Option<String>,
    pub oauth_refresh_token: Option<String>,
    pub oauth_token_expires_at: Option<OffsetDateTime>,
    pub provider_metadata: Option<serde_json::Value>,
    pub status: Option<ProviderStatus>,
    pub last_used_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderInfo {
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Vec<String>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: String,
    pub scope: Option<String>,
}

impl AuthProviderType {
    pub fn all() -> Vec<AuthProviderType> {
        vec![
            AuthProviderType::Email,
            AuthProviderType::Google,
            AuthProviderType::Github,
            AuthProviderType::Wallet,
        ]
    }

    pub fn requires_password(&self) -> bool {
        matches!(self, AuthProviderType::Email)
    }

    pub fn requires_oauth(&self) -> bool {
        matches!(self, AuthProviderType::Google | AuthProviderType::Github)
    }

    pub fn requires_wallet(&self) -> bool {
        matches!(self, AuthProviderType::Wallet)
    }

    pub fn oauth_scopes(&self) -> Vec<&'static str> {
        match self {
            AuthProviderType::Google => vec!["openid", "email", "profile"],
            AuthProviderType::Github => vec!["user:email", "read:user"],
            _ => vec![],
        }
    }
}

impl Display for AuthProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let provider_str = match self {
            AuthProviderType::Email => "email",
            AuthProviderType::Google => "google",
            AuthProviderType::Github => "github",
            AuthProviderType::Wallet => "wallet",
        };
        write!(f, "{}", provider_str)
    }
}

impl Display for ProviderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status_str = match self {
            ProviderStatus::Active => "active",
            ProviderStatus::Suspended => "suspended",
            ProviderStatus::Revoked => "revoked",
        };
        write!(f, "{}", status_str)
    }
}

impl UserAuthProvider {
    pub fn new_email_provider(
        user_id: Uuid,
        email: String,
        password_hash: String,
    ) -> UserAuthProviderForCreate {
        UserAuthProviderForCreate {
            user_id,
            provider_type: AuthProviderType::Email,
            provider_user_id: email.clone(),
            provider_email: Some(email),
            password_hash: Some(password_hash),
            wallet_address: None,
            public_key: None,
            oauth_access_token: None,
            oauth_refresh_token: None,
            oauth_token_expires_at: None,
            provider_metadata: None,
        }
    }

    pub fn new_wallet_provider(
        user_id: Uuid,
        wallet_address: String,
        public_key: String,
    ) -> UserAuthProviderForCreate {
        UserAuthProviderForCreate {
            user_id,
            provider_type: AuthProviderType::Wallet,
            provider_user_id: wallet_address.clone(),
            provider_email: None,
            password_hash: None,
            wallet_address: Some(wallet_address),
            public_key: Some(public_key),
            oauth_access_token: None,
            oauth_refresh_token: None,
            oauth_token_expires_at: None,
            provider_metadata: None,
        }
    }

    pub fn new_oauth_provider(
        user_id: Uuid,
        provider_type: AuthProviderType,
        provider_user_id: String,
        provider_email: String,
        oauth_data: OAuthTokenResponse,
        metadata: Option<serde_json::Value>,
    ) -> UserAuthProviderForCreate {
        let expires_at = oauth_data.expires_in.map(|expires_in| {
            OffsetDateTime::now_utc() + time::Duration::seconds(expires_in)
        });

        UserAuthProviderForCreate {
            user_id,
            provider_type,
            provider_user_id,
            provider_email: Some(provider_email),
            password_hash: None,
            wallet_address: None,
            public_key: None,
            oauth_access_token: Some(oauth_data.access_token),
            oauth_refresh_token: oauth_data.refresh_token,
            oauth_token_expires_at: expires_at,
            provider_metadata: metadata,
        }
    }

    pub fn is_oauth_token_expired(&self) -> bool {
        if let Some(expires_at) = self.oauth_token_expires_at {
            expires_at <= OffsetDateTime::now_utc()
        } else {
            false
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, ProviderStatus::Active)
    }

    pub fn validate_wallet_address(address: &str) -> bool {
        address.starts_with("0x") && address.len() == 66 && 
        address[2..].chars().all(|c| c.is_ascii_hexdigit())
    }
}