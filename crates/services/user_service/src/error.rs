use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;
use tracing::{error, warn};

// ============================================================================
// Error Types
// ============================================================================

#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, thiserror::Error)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // -- Client Errors (4xx)
    #[error("Bad request: {message}")]
    BadRequest { message: String },

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Access denied: {resource}")]
    AccessDenied { resource: String },

    #[error("Entity '{entity}' with id {id} not found")]
    EntityNotFound { entity: &'static str, id: i64 },

    #[error("Validation failed")]
    ValidationFailed {
        #[serde(flatten)]
        details: ValidationDetails,
    },

    #[error("Resource conflict: {message}")]
    Conflict { message: String },

    #[error("Rate limit exceeded for {resource}")]
    RateLimitExceeded { resource: String },

    // -- Server Errors (5xx)
    #[error("Internal server error")]
    InternalServerError {
        #[serde(skip)]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        #[serde(skip)]
        context: String,
    },

    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Database error")]
    DatabaseError {
        #[serde(skip)]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // -- Core Error Integration
    #[error(transparent)]
    CoreError(
        #[from]
        #[serde_as(as = "DisplayFromStr")]
        Arc<jd_core::Error>,
    ),
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Self::BadRequest { message } => Self::BadRequest {
                message: message.clone(),
            },
            Self::AuthenticationFailed { reason } => Self::AuthenticationFailed { reason: reason.clone() },
            Self::AccessDenied { resource } => Self::AccessDenied {
                resource: resource.clone(),
            },
            Self::EntityNotFound { entity, id } => Self::EntityNotFound {
                entity: *entity,
                id: *id,
            },
            Self::ValidationFailed { details } => Self::ValidationFailed {
                details: details.clone(),
            },
            Self::Conflict { message } => Self::Conflict {
                message: message.clone(),
            },
            Self::RateLimitExceeded { resource } => Self::RateLimitExceeded {
                resource: resource.clone(),
            },
            Self::InternalServerError { context, .. } => Self::InternalServerError {
                source: None,
                context: context.clone(),
            },
            Self::ServiceUnavailable { service } => Self::ServiceUnavailable {
                service: service.clone(),
            },
            Self::DatabaseError { .. } => Self::DatabaseError {
                source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "cloned error")),
            },
            Self::CoreError(err) => Self::CoreError(err.clone()),
        }
    }
}

// ============================================================================
// Validation Details
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
// Error Classification
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
// Error Extensions
// ============================================================================

impl Error {
    // -- Constructors with context
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest {
            message: message.into(),
        }
    }

    pub fn auth_failed(reason: impl Into<String>) -> Self {
        Self::AuthenticationFailed { reason: reason.into() }
    }

    pub fn access_denied(resource: impl Into<String>) -> Self {
        Self::AccessDenied {
            resource: resource.into(),
        }
    }

    pub fn not_found(entity: &'static str, id: i64) -> Self {
        Self::EntityNotFound { entity, id }
    }

    pub fn validation_failed(details: ValidationDetails) -> Self {
        Self::ValidationFailed { details }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    pub fn rate_limit(resource: impl Into<String>) -> Self {
        Self::RateLimitExceeded {
            resource: resource.into(),
        }
    }

    pub fn internal_error(context: impl Into<String>) -> Self {
        Self::InternalServerError {
            source: None,
            context: context.into(),
        }
    }

    pub fn internal_with_source(
        source: impl std::error::Error + Send + Sync + 'static,
        context: impl Into<String>,
    ) -> Self {
        Self::InternalServerError {
            source: Some(Box::new(source)),
            context: context.into(),
        }
    }

    pub fn service_unavailable(service: impl Into<String>) -> Self {
        Self::ServiceUnavailable {
            service: service.into(),
        }
    }

    pub fn database_error(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::DatabaseError {
            source: Box::new(source),
        }
    }

    // -- Error properties
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::ValidationFailed { .. } | Self::EntityNotFound { .. } => ErrorSeverity::Low,
            Self::BadRequest { .. } | Self::Conflict { .. } => ErrorSeverity::Medium,
            Self::AuthenticationFailed { .. } | Self::AccessDenied { .. } => ErrorSeverity::High,
            Self::InternalServerError { .. } | Self::DatabaseError { .. } => ErrorSeverity::Critical,
            Self::RateLimitExceeded { .. } | Self::ServiceUnavailable { .. } => ErrorSeverity::Medium,
            Self::CoreError(_) => ErrorSeverity::High,
        }
    }

    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::AuthenticationFailed { .. } => ErrorCategory::Authentication,
            Self::AccessDenied { .. } => ErrorCategory::Authorization,
            Self::ValidationFailed { .. } | Self::BadRequest { .. } => ErrorCategory::Validation,
            Self::EntityNotFound { .. } => ErrorCategory::NotFound,
            Self::Conflict { .. } => ErrorCategory::Conflict,
            Self::RateLimitExceeded { .. } => ErrorCategory::RateLimit,
            Self::InternalServerError { .. } | Self::DatabaseError { .. } => ErrorCategory::Internal,
            Self::ServiceUnavailable { .. } => ErrorCategory::External,
            Self::CoreError(_) => ErrorCategory::Internal,
        }
    }

    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::BadRequest { .. }
                | Self::AuthenticationFailed { .. }
                | Self::AccessDenied { .. }
                | Self::EntityNotFound { .. }
                | Self::ValidationFailed { .. }
                | Self::Conflict { .. }
                | Self::RateLimitExceeded { .. }
        )
    }

    pub fn is_server_error(&self) -> bool {
        !self.is_client_error()
    }

    pub fn should_log(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::High | ErrorSeverity::Critical)
    }

    // -- Error conversion
    pub fn client_status_and_error(&self, request_id: Option<String>) -> (StatusCode, ClientError) {
        let (status_code, error_code, message, details) = match self {
            // Client Errors (4xx)
            Self::BadRequest { message } => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST".to_string(),
                message.clone(),
                None,
            ),
            Self::AuthenticationFailed { reason } => (
                StatusCode::UNAUTHORIZED,
                "AUTHENTICATION_FAILED".to_string(),
                "Authentication required".to_string(),
                Some(serde_json::json!({ "reason": reason })),
            ),
            Self::AccessDenied { resource } => (
                StatusCode::FORBIDDEN,
                "ACCESS_DENIED".to_string(),
                "Access denied".to_string(),
                Some(serde_json::json!({ "resource": resource })),
            ),
            Self::EntityNotFound { entity, id } => (
                StatusCode::NOT_FOUND,
                "ENTITY_NOT_FOUND".to_string(),
                format!("{} not found", entity),
                Some(serde_json::json!({ "entity": entity, "id": id })),
            ),
            Self::ValidationFailed { details } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_FAILED".to_string(),
                "Validation failed".to_string(),
                Some(serde_json::to_value(details).unwrap_or_default()),
            ),
            Self::Conflict { message } => (StatusCode::CONFLICT, "CONFLICT".to_string(), message.clone(), None),
            Self::RateLimitExceeded { resource } => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED".to_string(),
                "Rate limit exceeded".to_string(),
                Some(serde_json::json!({ "resource": resource })),
            ),

            // Server Errors (5xx)
            Self::InternalServerError { context, .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR".to_string(),
                "Internal server error".to_string(),
                if cfg!(debug_assertions) {
                    Some(serde_json::json!({ "context": context }))
                } else {
                    None
                },
            ),
            Self::ServiceUnavailable { service } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE".to_string(),
                "Service temporarily unavailable".to_string(),
                Some(serde_json::json!({ "service": service })),
            ),
            Self::DatabaseError { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR".to_string(),
                "Database error occurred".to_string(),
                None,
            ),

            // Core Error Mapping
            Self::CoreError(core_err) => self.map_core_error(core_err),
        };

        let client_error = ClientError {
            error_code: error_code.to_string(),
            message,
            details,
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id,
        };

        (status_code, client_error)
    }

    fn map_core_error(&self, core_err: &jd_core::Error) -> (StatusCode, String, String, Option<serde_json::Value>) {
        match core_err {
            jd_core::Error::CantCreateModelManagerProvider(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERVICE_INITIALIZATION_ERROR".to_string(),
                "Service initialization failed".to_string(),
                None,
            ),
            jd_core::Error::ListLimitOverMax { max, actual } => (
                StatusCode::BAD_REQUEST,
                "LIST_LIMIT_EXCEEDED".to_string(),
                format!("List limit exceeded. Max: {}, Requested: {}", max, actual),
                Some(serde_json::json!({ "max": max, "actual": actual })),
            ),
            jd_core::Error::UniqueViolation { table, constraint } => (
                StatusCode::CONFLICT,
                "UNIQUE_VIOLATION".to_string(),
                "Resource already exists".to_string(),
                Some(serde_json::json!({ "table": table, "constraint": constraint })),
            ),
            jd_core::Error::CountFail => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "COUNT_OPERATION_FAILED".to_string(),
                "Count operation failed".to_string(),
                None,
            ),
            jd_core::Error::EntityNotFound { entity, id } => (
                StatusCode::NOT_FOUND,
                "ENTITY_NOT_FOUND".to_string(),
                format!("{} not found", entity),
                Some(serde_json::json!({ "entity": entity, "id": id })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "UNKNOWN_CORE_ERROR".to_string(),
                "Unknown core error occurred".to_string(),
                None,
            ),
        }
    }
}

// ============================================================================
// Axum Integration
// ============================================================================

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // Extract request ID from extensions if available
        let request_id = None; // TODO: Extract from request extensions

        let (status_code, client_error) = self.client_status_and_error(request_id);

        // Log based on severity
        match self.severity() {
            ErrorSeverity::Critical => {
                error!(
                    error = %self,
                    category = ?self.category(),
                    status_code = %status_code,
                    "Critical error occurred"
                );
            }
            ErrorSeverity::High => {
                error!(
                    error = %self,
                    category = ?self.category(),
                    status_code = %status_code,
                    "High severity error occurred"
                );
            }
            ErrorSeverity::Medium => {
                warn!(
                    error = %self,
                    category = ?self.category(),
                    status_code = %status_code,
                    "Medium severity error occurred"
                );
            }
            ErrorSeverity::Low => {
                // Only log in debug mode for low severity
                #[cfg(debug_assertions)]
                tracing::debug!(
                    error = %self,
                    category = ?self.category(),
                    status_code = %status_code,
                    "Low severity error occurred"
                );
            }
        }

        // Create response
        let mut response = (status_code, Json(client_error)).into_response();

        // Insert the original error for middleware/debugging
        response.extensions_mut().insert(self);

        response
    }
}

// ============================================================================
// Convenience From Implementations
// ============================================================================

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::database_error(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::internal_with_source(err, "JSON serialization/deserialization error")
    }
}

impl From<validator::ValidationErrors> for Error {
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
                    rejected_value: error.params.get("value").and_then(|v| serde_json::to_value(v).ok()),
                })
            })
            .collect();

        Self::validation_failed(ValidationDetails {
            field_errors,
            global_errors: vec![],
        })
    }
}

// ============================================================================
// Result Type Alias
// ============================================================================

// ============================================================================
// Testing Utilities
// ============================================================================

#[cfg(test)]
impl Error {
    pub fn assert_status(&self, expected: StatusCode) {
        let (actual, _) = self.client_status_and_error(None);
        assert_eq!(actual, expected, "Expected status {}, got {}", expected, actual);
    }

    pub fn assert_error_code(&self, expected: &str) {
        let (_, client_error) = self.client_status_and_error(None);
        assert_eq!(
            client_error.error_code, expected,
            "Expected error code {}, got {}",
            expected, client_error.error_code
        );
    }
}
