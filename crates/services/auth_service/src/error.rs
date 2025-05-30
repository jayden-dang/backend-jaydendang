use serde::{Deserialize, Serialize};
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
  pub error: String,
  pub code: String,
  pub details: Option<serde_json::Value>,
}

impl Error {
  pub fn new(error: &str, code: &str) -> Self {
    Self { error: error.to_string(), code: code.to_string(), details: None }
  }

  pub fn with_details(error: &str, code: &str, details: serde_json::Value) -> Self {
    Self { error: error.to_string(), code: code.to_string(), details: Some(details) }
  }

  // Nonce related errors
  pub fn nonce_not_found() -> Self {
    Self::new("Nonce not found", "NONCE_NOT_FOUND")
  }

  pub fn nonce_expired() -> Self {
    Self::new("Nonce has expired", "NONCE_EXPIRED")
  }

  pub fn invalid_signature() -> Self {
    Self::new("Invalid signature", "INVALID_SIGNATURE")
  }

  pub fn invalid_public_key() -> Self {
    Self::new("Invalid public key", "INVALID_PUBLIC_KEY")
  }

  // JWT related errors
  pub fn invalid_token() -> Self {
    Self::new("Invalid JWT token", "INVALID_TOKEN")
  }

  pub fn token_expired() -> Self {
    Self::new("JWT token has expired", "TOKEN_EXPIRED")
  }

  pub fn missing_auth_header() -> Self {
    Self::new("Missing Authorization header", "MISSING_AUTH_HEADER")
  }

  pub fn invalid_token_format() -> Self {
    Self::new("Invalid token format", "INVALID_TOKEN_FORMAT")
  }

  // Database related errors
  pub fn database_error(msg: &str) -> Self {
    Self::new(&format!("Database error: {}", msg), "DATABASE_ERROR")
  }

  // Redis related errors
  pub fn redis_error(msg: &str) -> Self {
    Self::new(&format!("Redis error: {}", msg), "REDIS_ERROR")
  }

  // Rate limiting errors
  pub fn rate_limit_exceeded() -> Self {
    Self::new("Rate limit exceeded", "RATE_LIMIT_EXCEEDED")
  }

  // Validation errors
  pub fn invalid_address() -> Self {
    Self::new("Invalid Sui address", "INVALID_ADDRESS")
  }

  pub fn invalid_request_data(field: &str) -> Self {
    Self::new(&format!("Invalid request data: {}", field), "INVALID_REQUEST_DATA")
  }

  // Internal errors
  pub fn internal_error(msg: &str) -> Self {
    Self::new(&format!("Internal error: {}", msg), "INTERNAL_ERROR")
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.error)
  }
}

impl std::error::Error for Error {}

// Convert from common error types
impl From<sqlx::Error> for Error {
  fn from(err: sqlx::Error) -> Self {
    Error::database_error(&err.to_string())
  }
}

impl From<redis::RedisError> for Error {
  fn from(err: redis::RedisError) -> Self {
    Error::redis_error(&err.to_string())
  }
}

impl From<jsonwebtoken::errors::Error> for Error {
  fn from(err: jsonwebtoken::errors::Error) -> Self {
    match err.kind() {
      jsonwebtoken::errors::ErrorKind::ExpiredSignature => Error::token_expired(),
      _ => Error::invalid_token(),
    }
  }
}

impl From<base64::DecodeError> for Error {
  fn from(_: base64::DecodeError) -> Self {
    Error::invalid_request_data("base64 decoding failed")
  }
}

// Axum response conversion
impl axum::response::IntoResponse for Error {
  fn into_response(self) -> axum::response::Response {
    let status = match self.code.as_str() {
      "NONCE_NOT_FOUND" | "NONCE_EXPIRED" | "INVALID_SIGNATURE" | "INVALID_PUBLIC_KEY" => {
        axum::http::StatusCode::UNAUTHORIZED
      }
      "INVALID_TOKEN" | "TOKEN_EXPIRED" | "MISSING_AUTH_HEADER" | "INVALID_TOKEN_FORMAT" => {
        axum::http::StatusCode::UNAUTHORIZED
      }
      "RATE_LIMIT_EXCEEDED" => axum::http::StatusCode::TOO_MANY_REQUESTS,
      "INVALID_ADDRESS" | "INVALID_REQUEST_DATA" => axum::http::StatusCode::BAD_REQUEST,
      _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    };

    let body = axum::Json(self);
    (status, body).into_response()
  }
}
