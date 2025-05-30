use async_trait::async_trait;

use crate::error::Result;

#[async_trait]
pub trait SignatureVerifier: Send + Sync {
  async fn verify_signature(
    &self,
    message: &str,
    signature: &str,
    public_key: &str,
    address: &str,
  ) -> Result<bool>;
}
