use time::OffsetDateTime;
use serde::{Deserialize, Serialize};
use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsString};
use sqlx::FromRow;

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
    Self {
      address,
      public_key,
      created_at: now,
      last_login: now,
      login_count: 1,
    }
  }

  pub fn update_login(&mut self) {
    self.last_login = OffsetDateTime::now_utc();
    self.login_count += 1;
  }

  /// Validate Sui address format (0x followed by 64 hex characters)
  pub fn is_valid_address(address: &str) -> bool {
    if !address.starts_with("0x") {
      return false;
    }
    
    let hex_part = &address[2..];
    hex_part.len() == 64 && hex_part.chars().all(|c| c.is_ascii_hexdigit())
  }

  /// Convert to create input
  pub fn into_create_input(self) -> AuthUserForCreate {
    AuthUserForCreate {
      address: self.address,
      public_key: self.public_key,
      created_at: self.created_at,
      last_login: self.last_login,
      login_count: self.login_count,
    }
  }

  /// Create update input for login update
  pub fn login_update_input(&self) -> AuthUserForUpdate {
    AuthUserForUpdate {
      public_key: Some(self.public_key.clone()),
      last_login: Some(OffsetDateTime::now_utc()),
      login_count: Some(self.login_count + 1),
    }
  }
} 