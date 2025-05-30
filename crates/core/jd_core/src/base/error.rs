use axum::{
  http::StatusCode,
  response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use std::fmt::{Debug, Display};
use tracing::{error, warn};

// ============================================================================
// Error Classification Traits and Types
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
  Low,      // Expected errors (validation, not found)
  Medium,   // Business logic errors
  High,     // System errors, database issues
  Critical, // Security issues, data corruption
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCategory {
  Authentication,
  Authorization,
  Validation,
  NotFound,
  Conflict,
  RateLimit,
  Internal,
  External,
}

// ============================================================================
// Core Error Trait
// ============================================================================

pub trait ServiceError: std::error::Error + Send + Sync + 'static {
  fn severity(&self) -> ErrorSeverity;
  fn category(&self) -> ErrorCategory;
  fn status_code(&self) -> StatusCode;
  fn error_code(&self) -> String;
  fn client_message(&self) -> String;
  fn details(&self) -> Option<serde_json::Value> {
    None
  }
  fn should_log(&self) -> bool {
    matches!(self.severity(), ErrorSeverity::High | ErrorSeverity::Critical)
  }
}

// ============================================================================
// Client Error Response
// ============================================================================

#[derive(Debug, Serialize, Clone)]
pub struct ClientError {
  pub error_code: String,
  pub message: String,
  pub details: Option<serde_json::Value>,
  pub timestamp: String,
  pub request_id: Option<String>,
}

// ============================================================================
// Generic Service Error Implementation
// ============================================================================

pub struct ServiceErrorWrapper<E: ServiceError> {
  pub error: E,
  pub request_id: Option<String>,
}

impl<E: ServiceError> IntoResponse for ServiceErrorWrapper<E> {
  fn into_response(self) -> Response {
    let status_code = self.error.status_code();
    let client_error = ClientError {
      error_code: self.error.error_code(),
      message: self.error.client_message(),
      details: self.error.details(),
      timestamp: chrono::Utc::now().to_rfc3339(),
      request_id: self.request_id,
    };

    // Log based on severity
    match self.error.severity() {
      ErrorSeverity::Critical => {
        error!(
            error = %self.error,
            category = ?self.error.category(),
            status_code = %status_code,
            "Critical error occurred"
        );
      }
      ErrorSeverity::High => {
        error!(
            error = %self.error,
            category = ?self.error.category(),
            status_code = %status_code,
            "High severity error occurred"
        );
      }
      ErrorSeverity::Medium => {
        warn!(
            error = %self.error,
            category = ?self.error.category(),
            status_code = %status_code,
            "Medium severity error occurred"
        );
      }
      ErrorSeverity::Low => {
        #[cfg(debug_assertions)]
        tracing::debug!(
            error = %self.error,
            category = ?self.error.category(),
            status_code = %status_code,
            "Low severity error occurred"
        );
      }
    }

    (status_code, Json(client_error)).into_response()
  }
}

// ============================================================================
// Validation Error Support
// ============================================================================

#[derive(Debug, Serialize, Clone)]
pub struct ValidationDetails {
  pub field_errors: Vec<FieldError>,
  pub global_errors: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FieldError {
  pub field: String,
  pub code: String,
  pub message: String,
  pub rejected_value: Option<serde_json::Value>,
}

impl From<validator::ValidationErrors> for ValidationDetails {
  fn from(err: validator::ValidationErrors) -> Self {
    let field_errors = err
      .field_errors()
      .iter()
      .flat_map(|(field, errors)| {
        let field = field.to_string();
        errors.iter().map(move |error| FieldError {
          field: field.clone(),
          code: error.code.to_string(),
          message: error
            .message
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| format!("Invalid {}", field)),
          rejected_value: error
            .params
            .get("value")
            .and_then(|v| serde_json::to_value(v).ok()),
        })
      })
      .collect();

    ValidationDetails { field_errors, global_errors: vec![] }
  }
}

// ============================================================================
// Error Mapping Trait
// ============================================================================

pub trait ErrorMapper<T, E: ServiceError> {
  fn map_service_error(self) -> Result<T, E>;
}

// ============================================================================
// Common Error Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct NotFoundError {
  pub entity: String,
  pub id: String,
}

impl Display for NotFoundError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} with id {} not found", self.entity, self.id)
  }
}

impl std::error::Error for NotFoundError {}

impl ServiceError for NotFoundError {
  fn severity(&self) -> ErrorSeverity {
    ErrorSeverity::Low
  }

  fn category(&self) -> ErrorCategory {
    ErrorCategory::NotFound
  }

  fn status_code(&self) -> StatusCode {
    StatusCode::NOT_FOUND
  }

  fn error_code(&self) -> String {
    "ENTITY_NOT_FOUND".to_string()
  }

  fn client_message(&self) -> String {
    format!("{} not found", self.entity)
  }

  fn details(&self) -> Option<serde_json::Value> {
    Some(serde_json::json!({
      "entity": self.entity,
      "id": self.id
    }))
  }
}

#[derive(Debug, Clone)]
pub struct ValidationError {
  pub details: ValidationDetails,
}

impl Display for ValidationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Validation failed")
  }
}

impl std::error::Error for ValidationError {}

impl ServiceError for ValidationError {
  fn severity(&self) -> ErrorSeverity {
    ErrorSeverity::Low
  }

  fn category(&self) -> ErrorCategory {
    ErrorCategory::Validation
  }

  fn status_code(&self) -> StatusCode {
    StatusCode::UNPROCESSABLE_ENTITY
  }

  fn error_code(&self) -> String {
    "VALIDATION_FAILED".to_string()
  }

  fn client_message(&self) -> String {
    "Validation failed".to_string()
  }

  fn details(&self) -> Option<serde_json::Value> {
    serde_json::to_value(&self.details).ok()
  }
}

#[derive(Debug, Clone)]
pub struct ConflictError {
  pub message: String,
}

impl Display for ConflictError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Conflict: {}", self.message)
  }
}

impl std::error::Error for ConflictError {}

impl ServiceError for ConflictError {
  fn severity(&self) -> ErrorSeverity {
    ErrorSeverity::Medium
  }

  fn category(&self) -> ErrorCategory {
    ErrorCategory::Conflict
  }

  fn status_code(&self) -> StatusCode {
    StatusCode::CONFLICT
  }

  fn error_code(&self) -> String {
    "CONFLICT".to_string()
  }

  fn client_message(&self) -> String {
    self.message.clone()
  }
}

#[derive(Debug, Clone)]
pub struct AuthenticationError {
  pub reason: String,
}

impl Display for AuthenticationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Authentication failed: {}", self.reason)
  }
}

impl std::error::Error for AuthenticationError {}

impl ServiceError for AuthenticationError {
  fn severity(&self) -> ErrorSeverity {
    ErrorSeverity::High
  }

  fn category(&self) -> ErrorCategory {
    ErrorCategory::Authentication
  }

  fn status_code(&self) -> StatusCode {
    StatusCode::UNAUTHORIZED
  }

  fn error_code(&self) -> String {
    "AUTHENTICATION_FAILED".to_string()
  }

  fn client_message(&self) -> String {
    "Authentication required".to_string()
  }

  fn details(&self) -> Option<serde_json::Value> {
    Some(serde_json::json!({ "reason": self.reason }))
  }
}

#[derive(Debug, Clone)]
pub struct AuthorizationError {
  pub resource: String,
}

impl Display for AuthorizationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Access denied to resource: {}", self.resource)
  }
}

impl std::error::Error for AuthorizationError {}

impl ServiceError for AuthorizationError {
  fn severity(&self) -> ErrorSeverity {
    ErrorSeverity::High
  }

  fn category(&self) -> ErrorCategory {
    ErrorCategory::Authorization
  }

  fn status_code(&self) -> StatusCode {
    StatusCode::FORBIDDEN
  }

  fn error_code(&self) -> String {
    "ACCESS_DENIED".to_string()
  }

  fn client_message(&self) -> String {
    "Access denied".to_string()
  }

  fn details(&self) -> Option<serde_json::Value> {
    Some(serde_json::json!({ "resource": self.resource }))
  }
}

// ============================================================================
// Error Helpers
// ============================================================================

pub fn not_found(entity: impl Into<String>, id: impl Into<String>) -> NotFoundError {
  NotFoundError { entity: entity.into(), id: id.into() }
}

pub fn validation_error(details: ValidationDetails) -> ValidationError {
  ValidationError { details }
}

pub fn conflict(message: impl Into<String>) -> ConflictError {
  ConflictError { message: message.into() }
}

pub fn auth_failed(reason: impl Into<String>) -> AuthenticationError {
  AuthenticationError { reason: reason.into() }
}

pub fn access_denied(resource: impl Into<String>) -> AuthorizationError {
  AuthorizationError { resource: resource.into() }
}
