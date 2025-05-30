use crate::error::{Error, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub address: String,
  pub public_key: String,
  pub token_type: String, // "access" or "refresh"
  pub exp: usize,         // Expiration timestamp
  pub iat: usize,         // Issued at timestamp
}

#[derive(Debug, Clone)]
pub struct JwtManager {
  secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
  pub access_token: String,
  pub refresh_token: String,
}

impl JwtManager {
  pub fn new(secret: String) -> Self {
    Self { secret }
  }

  /// Generate access and refresh tokens for a user
  pub fn generate_tokens(&self, address: &str, public_key: &str) -> Result<TokenPair> {
    let access_token = self.generate_access_token(address, public_key)?;
    let refresh_token = self.generate_refresh_token(address, public_key)?;

    Ok(TokenPair { access_token, refresh_token })
  }

  /// Generate an access token (1 hour expiry)
  fn generate_access_token(&self, address: &str, public_key: &str) -> Result<String> {
    let now = Utc::now();
    let exp = (now + Duration::hours(1)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
      address: address.to_string(),
      public_key: public_key.to_string(),
      token_type: "access".to_string(),
      exp,
      iat,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_ref()))
      .map_err(|e| Error::internal_error(&format!("Failed to generate access token: {}", e)))
  }

  /// Generate a refresh token (7 days expiry)
  fn generate_refresh_token(&self, address: &str, public_key: &str) -> Result<String> {
    let now = Utc::now();
    let exp = (now + Duration::days(7)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
      address: address.to_string(),
      public_key: public_key.to_string(),
      token_type: "refresh".to_string(),
      exp,
      iat,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_ref()))
      .map_err(|e| Error::internal_error(&format!("Failed to generate refresh token: {}", e)))
  }

  /// Validate and decode a token
  pub fn validate_token(&self, token: &str) -> Result<Claims> {
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(token, &DecodingKey::from_secret(self.secret.as_ref()), &validation)
      .map(|data| data.claims)
      .map_err(|e| e.into())
  }

  /// Generate a new access token from a valid refresh token
  pub fn refresh_access_token(&self, refresh_token: &str) -> Result<String> {
    let claims = self.validate_token(refresh_token)?;

    // Ensure it's a refresh token
    if claims.token_type != "refresh" {
      return Err(Error::invalid_token());
    }

    self.generate_access_token(&claims.address, &claims.public_key)
  }

  /// Extract token from Authorization header
  pub fn extract_token_from_header(auth_header: &str) -> Result<&str> {
    if !auth_header.starts_with("Bearer ") {
      return Err(Error::invalid_token_format());
    }

    Ok(&auth_header[7..]) // Skip "Bearer "
  }
}
