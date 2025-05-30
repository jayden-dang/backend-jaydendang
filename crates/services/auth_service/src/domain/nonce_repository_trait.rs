use async_trait::async_trait;

use crate::domain::Nonce;
use crate::error::Result;

#[async_trait]
pub trait NonceRepository: Send + Sync {
  async fn store_nonce(&self, nonce: &Nonce) -> Result<()>;
  async fn get_nonce(&self, address: &str) -> Result<Option<Nonce>>;
  async fn remove_nonce(&self, address: &str) -> Result<()>;
}
