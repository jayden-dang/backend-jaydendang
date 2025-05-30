use tracing::info;

use crate::domain::JwtManager;
use crate::error::Result;

pub struct RefreshTokenUseCase {
  jwt_manager: JwtManager,
}

impl RefreshTokenUseCase {
  pub fn new(jwt_secret: String) -> Self {
    Self { jwt_manager: JwtManager::new(jwt_secret) }
  }

  pub async fn execute(&self, refresh_token: &str) -> Result<String> {
    info!("🔄 Refreshing access token");

    // Validate refresh token and generate new access token
    let access_token = self.jwt_manager.refresh_access_token(refresh_token)?;

    info!("✅ Access token refreshed successfully");
    Ok(access_token)
  }
}
