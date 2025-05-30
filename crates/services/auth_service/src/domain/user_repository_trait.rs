use async_trait::async_trait;

use crate::domain::AuthUser;
use crate::error::Result;

#[async_trait]
pub trait UserRepository: Send + Sync {
  async fn create_user(&self, user: &AuthUser) -> Result<()>;
  async fn get_user(&self, address: &str) -> Result<Option<AuthUser>>;
  async fn update_user(&self, user: &AuthUser) -> Result<()>;
}
