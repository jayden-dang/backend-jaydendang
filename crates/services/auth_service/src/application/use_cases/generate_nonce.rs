use crate::domain::{AuthUser, Nonce, NonceRepository};
use crate::error::{Error, Result};

pub struct GenerateNonceUseCase<R: NonceRepository> {
  repository: R,
}

impl<R: NonceRepository> GenerateNonceUseCase<R> {
  pub fn new(repository: R) -> Self {
    Self { repository }
  }

  pub async fn execute(&self, address: &str) -> Result<Nonce> {
    // Validate address format
    if !AuthUser::is_valid_address(address) {
      return Err(Error::invalid_address());
    }

    // Generate new nonce
    let nonce = Nonce::generate(address.to_string());

    // Store nonce in repository
    self.repository.store_nonce(&nonce).await?;

    Ok(nonce)
  }
}
