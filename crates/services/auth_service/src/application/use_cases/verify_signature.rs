use tracing::{error, info, warn};

use crate::domain::{
  AuthUser, JwtManager, NonceRepository, SignatureVerifier, TokenPair, UserRepository,
};
use crate::error::{Error, Result};

pub struct VerifySignatureUseCase<N: NonceRepository, U: UserRepository, S: SignatureVerifier> {
  nonce_repo: N,
  user_repo: U,
  signature_verifier: S,
  jwt_manager: JwtManager,
}

impl<N: NonceRepository, U: UserRepository, S: SignatureVerifier> VerifySignatureUseCase<N, U, S> {
  pub fn new(nonce_repo: N, user_repo: U, signature_verifier: S, jwt_secret: String) -> Self {
    Self { nonce_repo, user_repo, signature_verifier, jwt_manager: JwtManager::new(jwt_secret) }
  }

  pub async fn execute(
    &self,
    address: &str,
    signature: &str,
    public_key: &str,
  ) -> Result<(AuthUser, TokenPair)> {
    info!("🚀 Starting signature verification for address: {}", address);

    // Validate address format
    if !AuthUser::is_valid_address(address) {
      error!("❌ Invalid address format: {}", address);
      return Err(Error::invalid_address());
    }

    // Get stored nonce
    let nonce = self.nonce_repo.get_nonce(address).await?.ok_or_else(|| {
      error!("❌ Nonce not found for address: {}", address);
      Error::nonce_not_found()
    })?;

    info!("✅ Nonce found for address: {}", address);

    // Check if nonce has expired
    if nonce.is_expired() {
      warn!("⚠️ Nonce expired for address: {}", address);
      self.nonce_repo.remove_nonce(address).await?;
      return Err(Error::nonce_expired());
    }

    // Get the message that should have been signed
    let message = nonce.get_signing_message();
    info!("📝 Expected message: {}", message);

    // Verify signature
    let is_valid = self
      .signature_verifier
      .verify_signature(&message, signature, public_key, address)
      .await?;

    if !is_valid {
      error!("❌ Signature verification failed for address: {}", address);
      return Err(Error::invalid_signature());
    }

    info!("✅ Signature verified successfully for address: {}", address);

    // Remove used nonce
    self.nonce_repo.remove_nonce(address).await?;
    info!("🗑️ Used nonce removed for address: {}", address);

    // Get or create user
    let user = match self.user_repo.get_user(address).await? {
      Some(mut existing_user) => {
        info!("👤 Existing user found, updating login info");
        // Update login info
        existing_user.update_login();
        existing_user.public_key = public_key.to_string(); // Update public key if changed
        self.user_repo.update_user(&existing_user).await?;
        existing_user
      }
      None => {
        info!("👤 Creating new user");
        // Create new user
        let new_user = AuthUser::new(address.to_string(), public_key.to_string());
        self.user_repo.create_user(&new_user).await?;
        new_user
      }
    };

    // Generate JWT tokens
    let tokens = self
      .jwt_manager
      .generate_tokens(&user.address, &user.public_key)?;

    info!("🎉 Authentication successful for address: {}", address);
    Ok((user, tokens))
  }
}
