use crate::domain::{AuthUser, TokenPair};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct NonceResponse {
  pub nonce: String,
  pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
  pub address: String,
  pub public_key: String,
  #[serde(with = "time::serde::rfc3339")]
  pub created_at: OffsetDateTime,
  #[serde(with = "time::serde::rfc3339")]
  pub last_login: OffsetDateTime,
  pub login_count: i32,
}

impl From<AuthUser> for UserInfo {
  fn from(user: AuthUser) -> Self {
    Self {
      address: user.address,
      public_key: user.public_key,
      created_at: user.created_at,
      last_login: user.last_login,
      login_count: user.login_count,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
  pub success: bool,
  pub user: UserInfo,
  pub tokens: TokenPair,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshResponse {
  pub access_token: String,
}
