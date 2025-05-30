use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NonceRequest {
  #[validate(length(min = 66, max = 66, message = "Address must be 66 characters (0x + 64 hex)"))]
  #[validate(custom(function = "validate_sui_address"))]
  pub address: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct VerifyRequest {
  #[validate(length(min = 66, max = 66, message = "Address must be 66 characters"))]
  #[validate(custom(function = "validate_sui_address"))]
  pub address: String,

  #[validate(length(min = 1, message = "Signature cannot be empty"))]
  pub signature: String,

  #[validate(length(min = 1, message = "Public key cannot be empty"))]
  pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RefreshRequest {
  #[validate(length(min = 1, message = "Refresh token cannot be empty"))]
  pub refresh_token: String,
}

fn validate_sui_address(address: &str) -> Result<(), validator::ValidationError> {
  if crate::domain::AuthUser::is_valid_address(address) {
    Ok(())
  } else {
    Err(validator::ValidationError::new("invalid_sui_address"))
  }
}
