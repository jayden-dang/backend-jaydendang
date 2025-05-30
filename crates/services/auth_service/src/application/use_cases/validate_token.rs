use tracing::{error, info};

use crate::domain::{AuthUser, JwtManager, UserRepository};
use crate::error::{Error, Result};

pub struct ValidateTokenUseCase<R: UserRepository> {
  user_repo: R,
  jwt_manager: JwtManager,
}

impl<R: UserRepository> ValidateTokenUseCase<R> {
  pub fn new(user_repo: R, jwt_secret: String) -> Self {
    Self { user_repo, jwt_manager: JwtManager::new(jwt_secret) }
  }

  pub async fn execute(&self, token: &str) -> Result<AuthUser> {
    info!("🔍 Validating access token");

    // Validate token and extract claims
    let claims = self.jwt_manager.validate_token(token)?;

    // Ensure it's an access token
    if claims.token_type != "access" {
      error!("❌ Invalid token type: {} (expected access)", claims.token_type);
      return Err(Error::invalid_token());
    }

    // Get user from database
    let user = self
      .user_repo
      .get_user(&claims.address)
      .await?
      .ok_or_else(|| {
        error!("❌ User not found for address: {}", claims.address);
        Error::invalid_token()
      })?;

    // Verify public key matches
    if user.public_key != claims.public_key {
      error!("❌ Public key mismatch for address: {}", claims.address);
      return Err(Error::invalid_token());
    }

    info!("✅ Access token validated successfully for address: {}", claims.address);
    Ok(user)
  }

  pub fn extract_token_from_header(auth_header: &str) -> Result<&str> {
    JwtManager::extract_token_from_header(auth_header)
  }
}
