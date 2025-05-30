use tracing::{error, info, warn};

use crate::domain::{AuthUser, JwtManager, Nonce, TokenPair};
use crate::error::{Error, Result};
use crate::infrastructure::{NonceRepository, SignatureVerifier, UserRepository};
use std::sync::Arc;

pub struct AuthService {
  nonce_repo: Arc<dyn NonceRepository>,
  user_repo: Arc<dyn UserRepository>,
  signature_verifier: Arc<dyn SignatureVerifier>,
  jwt_manager: JwtManager,
}

impl AuthService {
  pub fn new(
    nonce_repo: Arc<dyn NonceRepository>,
    user_repo: Arc<dyn UserRepository>,
    signature_verifier: Arc<dyn SignatureVerifier>,
    jwt_secret: String,
  ) -> Self {
    Self { nonce_repo, user_repo, signature_verifier, jwt_manager: JwtManager::new(jwt_secret) }
  }

  /// Generate a nonce for wallet authentication
  pub async fn generate_nonce(&self, address: &str) -> Result<Nonce> {
    // Validate address format
    if !AuthUser::is_valid_address(address) {
      return Err(Error::invalid_address());
    }

    // Generate new nonce
    let nonce = Nonce::generate(address.to_string());

    // Store nonce in repository
    self.nonce_repo.store_nonce(&nonce).await?;

    Ok(nonce)
  }

  /// Verify wallet signature and authenticate user
  pub async fn verify_signature(
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

  /// Extract token from Authorization header
  pub fn extract_token_from_header<'a>(&self, auth_header: &'a str) -> Result<&'a str> {
    JwtManager::extract_token_from_header(auth_header)
  }

  /// Refresh access token using refresh token
  pub async fn refresh_token(&self, refresh_token: &str) -> Result<String> {
    info!("🔄 Refreshing access token");

    // Validate refresh token and generate new access token
    let access_token = self.jwt_manager.refresh_access_token(refresh_token)?;

    info!("✅ Access token refreshed successfully");
    Ok(access_token)
  }

  /// Validate access token and return user info
  pub async fn validate_access_token(&self, token: &str) -> Result<AuthUser> {
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
}
