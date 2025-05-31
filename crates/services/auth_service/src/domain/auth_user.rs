use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsString, OpValsValue};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use super::user_role::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
pub struct UnifiedAuthUser {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub username: String,
    pub display_name: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
    pub is_email_verified: bool,
    pub is_profile_complete: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_login: Option<OffsetDateTime>,
    pub login_count: i32,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct UnifiedAuthUserForCreate {
    pub email: Option<String>,
    pub username: String,
    pub display_name: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
    pub is_email_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct UnifiedAuthUserForUpdate {
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
    pub is_email_verified: Option<bool>,
    pub is_profile_complete: Option<bool>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_login: Option<OffsetDateTime>,
    pub login_count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, FilterNodes)]
pub struct UnifiedAuthUserFilter {
    pub user_id: Option<OpValsValue>,
    pub email: Option<OpValsString>,
    pub username: Option<OpValsString>,
    pub role: Option<OpValsString>,
    pub is_active: Option<OpValsValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermission {
    pub permission_name: String,
    pub resource: String,
    pub action: String,
}

impl UnifiedAuthUser {
    pub fn new(username: String, email: Option<String>) -> UnifiedAuthUserForCreate {
        UnifiedAuthUserForCreate {
            email,
            username,
            display_name: None,
            role: Some(UserRole::Normal),
            is_active: Some(true),
            is_email_verified: Some(false),
        }
    }

    pub fn update_login(&self) -> UnifiedAuthUserForUpdate {
        UnifiedAuthUserForUpdate {
            email: None,
            username: None,
            display_name: None,
            role: None,
            is_active: None,
            is_email_verified: None,
            is_profile_complete: None,
            last_login: Some(OffsetDateTime::now_utc()),
            login_count: Some(self.login_count + 1),
        }
    }

    pub fn can_access_role(&self, required_role: UserRole) -> bool {
        self.is_active && self.role.can_access(required_role)
    }

    pub fn is_soft_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn validate_username(username: &str) -> bool {
        username.len() >= 3 && 
        username.len() <= 50 && 
        username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }

    pub fn validate_email(email: &str) -> bool {
        email.contains('@') && email.len() <= 255
    }
}

// Legacy struct for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
pub struct AuthUser {
    pub address: String,
    pub public_key: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_login: OffsetDateTime,
    pub login_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct AuthUserForCreate {
    pub address: String,
    pub public_key: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub last_login: OffsetDateTime,
    pub login_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Fields)]
pub struct AuthUserForUpdate {
    pub public_key: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_login: Option<OffsetDateTime>,
    pub login_count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, FilterNodes)]
pub struct AuthUserFilter {
    pub address: Option<OpValsString>,
}

impl AuthUser {
    pub fn new(address: String, public_key: String) -> Self {
        let now = OffsetDateTime::now_utc();
        Self { address, public_key, created_at: now, last_login: now, login_count: 1 }
    }

    pub fn update_login(&mut self) {
        self.last_login = OffsetDateTime::now_utc();
        self.login_count += 1;
    }

    pub fn is_valid_address(address: &str) -> bool {
        if !address.starts_with("0x") {
            return false;
        }

        let hex_part = &address[2..];
        hex_part.len() == 64 && hex_part.chars().all(|c| c.is_ascii_hexdigit())
    }

    pub fn into_create_input(self) -> AuthUserForCreate {
        AuthUserForCreate {
            address: self.address,
            public_key: self.public_key,
            created_at: self.created_at,
            last_login: self.last_login,
            login_count: self.login_count,
        }
    }

    pub fn login_update_input(&self) -> AuthUserForUpdate {
        AuthUserForUpdate {
            public_key: Some(self.public_key.clone()),
            last_login: Some(OffsetDateTime::now_utc()),
            login_count: Some(self.login_count + 1),
        }
    }
}
