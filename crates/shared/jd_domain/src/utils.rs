#![allow(unused)]
// ============================================================================
// CUSTOM VALIDATORS - Domain-specific validation logic
// ============================================================================

use validator::ValidationError;

/// Custom validator for birth year
pub(crate) fn validate_birth_year(year: i32) -> Result<(), ValidationError> {
  let current_year = 2024; // In real app, get from time service
  if year < 1900 || year > current_year - 13 {
    return Err(ValidationError::new("invalid_birth_year"));
  }
  Ok(())
}

/// Custom validator for username format (alphanumeric + underscore only)
pub(crate) fn validate_username_format(username: &str) -> Result<(), ValidationError> {
  if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
    return Err(ValidationError::new("invalid_username_format"));
  }
  Ok(())
}

/// Custom validator for timezone format
pub(crate) fn validate_timezone(timezone: &str) -> Result<(), ValidationError> {
  // Basic timezone validation - in real app, use chrono-tz
  if !timezone.contains('/') || timezone.len() < 3 {
    return Err(ValidationError::new("invalid_timezone"));
  }
  Ok(())
}

/// Custom validator for country code (ISO 3166-1 alpha-2)
pub(crate) fn validate_country_code(code: &str) -> Result<(), ValidationError> {
  if code.len() != 2 || !code.chars().all(|c| c.is_alphabetic() && c.is_uppercase()) {
    return Err(ValidationError::new("invalid_country_code"));
  }
  Ok(())
}

/// Custom validator for bio length (not too long)
pub(crate) fn validate_bio_length(bio: &str) -> Result<(), ValidationError> {
  if bio.len() > 500 {
    return Err(ValidationError::new("bio_too_long"));
  }
  Ok(())
}
