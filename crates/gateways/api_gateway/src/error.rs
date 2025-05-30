use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_with::serde_as;
use tracing::{error, info, warn};

use crate::middleware::{self};

// ============================================================================
// Error Types
// ============================================================================

#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, thiserror::Error)]
#[serde(tag = "type", content = "data")]
pub enum Error {
  // -- Request/Response Errors
  #[error("Request timestamp not found in request extensions")]
  ReqStampNotInReqExt,

  #[error("Invalid request format: {message}")]
  InvalidRequestFormat { message: String },

  #[error("Request body too large: {size} bytes (max: {max_size})")]
  RequestTooLarge { size: u64, max_size: u64 },

  #[error("Missing required header: {header}")]
  MissingRequiredHeader { header: String },

  #[error("Invalid header value for '{header}': {value}")]
  InvalidHeaderValue { header: String, value: String },

  // -- Authentication & Authorization
  #[error("Authentication context error")]
  CtxExt(#[from] middleware::mw_auth::CtxExtError),

  #[error("API key authentication failed: {reason}")]
  ApiKeyAuthFailed { reason: String },

  #[error("JWT token validation failed: {reason}")]
  JwtValidationFailed { reason: String },

  #[error("Session expired at {expired_at}")]
  SessionExpired { expired_at: String },

  #[error("Insufficient permissions for resource '{resource}'")]
  InsufficientPermissions { resource: String },

  #[error("Rate limit exceeded for client '{client_id}': {limit} requests per {window}")]
  RateLimitExceeded { client_id: String, limit: u32, window: String },

  // -- Service Communication
  #[error("Downstream service '{service}' unavailable")]
  ServiceUnavailable { service: String },

  #[error("Service '{service}' timeout after {timeout_ms}ms")]
  ServiceTimeout { service: String, timeout_ms: u64 },

  #[error("Service '{service}' returned error: {status_code}")]
  Servic {
    service: String,
    status_code: u16,
    #[serde(skip)]
    body: Option<String>,
  },

  #[error("Circuit breaker open for service '{service}'")]
  CircuitBreakerOpen { service: String },

  #[error("Load balancer has no healthy instances for service '{service}'")]
  NoHealthyInstances { service: String },

  // -- Gateway Specific
  #[error("Route not found for path '{path}' and method '{method}'")]
  RouteNotFound { path: String, method: String },

  #[error("Service discovery failed for service '{service}'")]
  ServiceDiscoveryFailed { service: String },

  #[error("Request routing failed: {reason}")]
  RoutingFailed { reason: String },

  #[error("Gateway configuration error: {config_key}")]
  GatewayConfig { config_key: String },

  // -- Monitoring & Observability
  #[error("Request correlation ID not found")]
  MissingCorrelationId,

  #[error("Tracing context lost")]
  TracingContextLost,

  #[error("Metrics collection failed: {metric_name}")]
  MetricsCollectionFailed { metric_name: String },

  // -- Cache & State
  #[error("Cache operation failed for key '{key}'")]
  CacheOperationFailed { key: String },

  #[error("Distributed lock acquisition failed for resource '{resource}'")]
  LockAcquisitionFailed { resource: String },

  #[error("Session store operation failed")]
  SessionStore {
    #[serde(skip)]
    source: Box<dyn std::error::Error + Send + Sync>,
  },

  // -- External Dependencies
  #[error("Database connection pool exhausted")]
  DatabasePoolExhausted,

  #[error("Redis connection failed")]
  RedisConnectionFailed {
    #[serde(skip)]
    source: Box<dyn std::error::Error + Send + Sync>,
  },

  #[error("Message queue operation failed")]
  MessageQueue { queue: String, operation: String },

  // -- Security
  #[error("Suspicious request detected: {reason}")]
  SuspiciousRequest { reason: String },

  #[error("CORS policy violation from origin '{origin}'")]
  CorsViolation { origin: String },

  #[error("Content Security Policy violation")]
  CspViolation { directive: String },

  #[error("Request blocked by security policy: {policy}")]
  SecurityPolicyViolation { policy: String },
}

impl Clone for Error {
  fn clone(&self) -> Self {
    match self {
      Self::SessionStore { .. } => {
        Self::SessionStore { source: Box::new(std::io::Error::other("cloned error")) }
      }
      Self::RedisConnectionFailed { .. } => {
        Self::RedisConnectionFailed { source: Box::new(std::io::Error::other("cloned error")) }
      }
      Self::ReqStampNotInReqExt => Self::ReqStampNotInReqExt,
      Self::InvalidRequestFormat { message } => {
        Self::InvalidRequestFormat { message: message.clone() }
      }
      Self::RequestTooLarge { size, max_size } => {
        Self::RequestTooLarge { size: *size, max_size: *max_size }
      }
      Self::MissingRequiredHeader { header } => {
        Self::MissingRequiredHeader { header: header.clone() }
      }
      Self::InvalidHeaderValue { header, value } => {
        Self::InvalidHeaderValue { header: header.clone(), value: value.clone() }
      }
      Self::CtxExt(e) => Self::CtxExt(e.clone()),
      Self::ApiKeyAuthFailed { reason } => Self::ApiKeyAuthFailed { reason: reason.clone() },
      Self::JwtValidationFailed { reason } => Self::JwtValidationFailed { reason: reason.clone() },
      Self::SessionExpired { expired_at } => {
        Self::SessionExpired { expired_at: expired_at.clone() }
      }
      Self::InsufficientPermissions { resource } => {
        Self::InsufficientPermissions { resource: resource.clone() }
      }
      Self::RateLimitExceeded { client_id, limit, window } => Self::RateLimitExceeded {
        client_id: client_id.clone(),
        limit: *limit,
        window: window.clone(),
      },
      Self::ServiceUnavailable { service } => Self::ServiceUnavailable { service: service.clone() },
      Self::ServiceTimeout { service, timeout_ms } => {
        Self::ServiceTimeout { service: service.clone(), timeout_ms: *timeout_ms }
      }
      Self::Servic { service, status_code, body } => {
        Self::Servic { service: service.clone(), status_code: *status_code, body: body.clone() }
      }
      Self::CircuitBreakerOpen { service } => Self::CircuitBreakerOpen { service: service.clone() },
      Self::NoHealthyInstances { service } => Self::NoHealthyInstances { service: service.clone() },
      Self::RouteNotFound { path, method } => {
        Self::RouteNotFound { path: path.clone(), method: method.clone() }
      }
      Self::ServiceDiscoveryFailed { service } => {
        Self::ServiceDiscoveryFailed { service: service.clone() }
      }
      Self::RoutingFailed { reason } => Self::RoutingFailed { reason: reason.clone() },
      Self::GatewayConfig { config_key } => Self::GatewayConfig { config_key: config_key.clone() },
      Self::MissingCorrelationId => Self::MissingCorrelationId,
      Self::TracingContextLost => Self::TracingContextLost,
      Self::MetricsCollectionFailed { metric_name } => {
        Self::MetricsCollectionFailed { metric_name: metric_name.clone() }
      }
      Self::CacheOperationFailed { key } => Self::CacheOperationFailed { key: key.clone() },
      Self::LockAcquisitionFailed { resource } => {
        Self::LockAcquisitionFailed { resource: resource.clone() }
      }
      Self::DatabasePoolExhausted => Self::DatabasePoolExhausted,
      Self::MessageQueue { queue, operation } => {
        Self::MessageQueue { queue: queue.clone(), operation: operation.clone() }
      }
      Self::SuspiciousRequest { reason } => Self::SuspiciousRequest { reason: reason.clone() },
      Self::CorsViolation { origin } => Self::CorsViolation { origin: origin.clone() },
      Self::CspViolation { directive } => Self::CspViolation { directive: directive.clone() },
      Self::SecurityPolicyViolation { policy } => {
        Self::SecurityPolicyViolation { policy: policy.clone() }
      }
    }
  }
}

// ============================================================================
// Error Classification
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
  Low,      // Expected errors (auth failures, validation)
  Medium,   // Business logic errors, service issues
  High,     // Infrastructure problems, security issues
  Critical, // Gateway failures, data corruption
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCategory {
  Authentication,
  Authorization,
  Validation,
  Routing,
  ServiceCommunication,
  Infrastructure,
  Security,
  Monitoring,
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
  pub retry_after: Option<u32>, // For rate limiting
  pub trace_id: Option<String>,
}

// ============================================================================
// Error Extensions & Analysis
// ============================================================================

impl Error {
  // -- Constructor methods
  pub fn invalid_request(message: impl Into<String>) -> Self {
    Self::InvalidRequestFormat { message: message.into() }
  }

  pub fn request_too_large(size: u64, max_size: u64) -> Self {
    Self::RequestTooLarge { size, max_size }
  }

  pub fn missing_header(header: impl Into<String>) -> Self {
    Self::MissingRequiredHeader { header: header.into() }
  }

  pub fn invalid_header(header: impl Into<String>, value: impl Into<String>) -> Self {
    Self::InvalidHeaderValue { header: header.into(), value: value.into() }
  }

  pub fn api_key_failed(reason: impl Into<String>) -> Self {
    Self::ApiKeyAuthFailed { reason: reason.into() }
  }

  pub fn jwt_failed(reason: impl Into<String>) -> Self {
    Self::JwtValidationFailed { reason: reason.into() }
  }

  pub fn session_expired(expired_at: impl Into<String>) -> Self {
    Self::SessionExpired { expired_at: expired_at.into() }
  }

  pub fn insufficient_permissions(resource: impl Into<String>) -> Self {
    Self::InsufficientPermissions { resource: resource.into() }
  }

  pub fn rate_limited(client_id: impl Into<String>, limit: u32, window: impl Into<String>) -> Self {
    Self::RateLimitExceeded { client_id: client_id.into(), limit, window: window.into() }
  }

  pub fn service_unavailable(service: impl Into<String>) -> Self {
    Self::ServiceUnavailable { service: service.into() }
  }

  pub fn service_timeout(service: impl Into<String>, timeout_ms: u64) -> Self {
    Self::ServiceTimeout { service: service.into(), timeout_ms }
  }

  pub fn service_error(service: impl Into<String>, status_code: u16, body: Option<String>) -> Self {
    Self::Servic { service: service.into(), status_code, body }
  }

  pub fn circuit_breaker_open(service: impl Into<String>) -> Self {
    Self::CircuitBreakerOpen { service: service.into() }
  }

  pub fn route_not_found(path: impl Into<String>, method: impl Into<String>) -> Self {
    Self::RouteNotFound { path: path.into(), method: method.into() }
  }

  pub fn routing_failed(reason: impl Into<String>) -> Self {
    Self::RoutingFailed { reason: reason.into() }
  }

  pub fn suspicious_request(reason: impl Into<String>) -> Self {
    Self::SuspiciousRequest { reason: reason.into() }
  }

  // -- Error properties
  pub fn severity(&self) -> ErrorSeverity {
    match self {
      // Low severity - expected errors
      Self::CtxExt(_)
      | Self::ApiKeyAuthFailed { .. }
      | Self::JwtValidationFailed { .. }
      | Self::SessionExpired { .. }
      | Self::InsufficientPermissions { .. }
      | Self::InvalidRequestFormat { .. }
      | Self::MissingRequiredHeader { .. }
      | Self::InvalidHeaderValue { .. }
      | Self::RouteNotFound { .. } => ErrorSeverity::Low,

      // Medium severity - business/service issues
      Self::RateLimitExceeded { .. }
      | Self::ServiceTimeout { .. }
      | Self::Servic { .. }
      | Self::RequestTooLarge { .. }
      | Self::CacheOperationFailed { .. }
      | Self::MessageQueue { .. } => ErrorSeverity::Medium,

      // High severity - infrastructure/security
      Self::ServiceUnavailable { .. }
      | Self::CircuitBreakerOpen { .. }
      | Self::NoHealthyInstances { .. }
      | Self::ServiceDiscoveryFailed { .. }
      | Self::DatabasePoolExhausted
      | Self::RedisConnectionFailed { .. }
      | Self::SuspiciousRequest { .. }
      | Self::SecurityPolicyViolation { .. } => ErrorSeverity::High,

      // Critical severity - gateway failures
      Self::ReqStampNotInReqExt
      | Self::GatewayConfig { .. }
      | Self::RoutingFailed { .. }
      | Self::MissingCorrelationId
      | Self::TracingContextLost
      | Self::SessionStore { .. }
      | Self::LockAcquisitionFailed { .. } => ErrorSeverity::Critical,

      // Security violations
      Self::CorsViolation { .. } | Self::CspViolation { .. } => ErrorSeverity::High,

      Self::MetricsCollectionFailed { .. } => ErrorSeverity::Low,
    }
  }

  pub fn category(&self) -> ErrorCategory {
    match self {
      Self::CtxExt(_)
      | Self::ApiKeyAuthFailed { .. }
      | Self::JwtValidationFailed { .. }
      | Self::SessionExpired { .. } => ErrorCategory::Authentication,

      Self::InsufficientPermissions { .. } | Self::RateLimitExceeded { .. } => {
        ErrorCategory::Authorization
      }

      Self::InvalidRequestFormat { .. }
      | Self::RequestTooLarge { .. }
      | Self::MissingRequiredHeader { .. }
      | Self::InvalidHeaderValue { .. } => ErrorCategory::Validation,

      Self::RouteNotFound { .. }
      | Self::RoutingFailed { .. }
      | Self::ServiceDiscoveryFailed { .. } => ErrorCategory::Routing,

      Self::ServiceUnavailable { .. }
      | Self::ServiceTimeout { .. }
      | Self::Servic { .. }
      | Self::CircuitBreakerOpen { .. }
      | Self::NoHealthyInstances { .. } => ErrorCategory::ServiceCommunication,

      Self::DatabasePoolExhausted
      | Self::RedisConnectionFailed { .. }
      | Self::MessageQueue { .. }
      | Self::CacheOperationFailed { .. }
      | Self::SessionStore { .. }
      | Self::LockAcquisitionFailed { .. } => ErrorCategory::Infrastructure,

      Self::SuspiciousRequest { .. }
      | Self::CorsViolation { .. }
      | Self::CspViolation { .. }
      | Self::SecurityPolicyViolation { .. } => ErrorCategory::Security,

      Self::MissingCorrelationId
      | Self::TracingContextLost
      | Self::MetricsCollectionFailed { .. }
      | Self::ReqStampNotInReqExt => ErrorCategory::Monitoring,

      Self::GatewayConfig { .. } => ErrorCategory::Infrastructure,
    }
  }

  pub fn is_retryable(&self) -> bool {
    matches!(
      self,
      Self::ServiceTimeout { .. }
        | Self::ServiceUnavailable { .. }
        | Self::DatabasePoolExhausted
        | Self::RedisConnectionFailed { .. }
        | Self::NoHealthyInstances { .. }
        | Self::CacheOperationFailed { .. }
    )
  }

  pub fn should_log(&self) -> bool {
    !matches!(self.severity(), ErrorSeverity::Low)
  }

  pub fn retry_after_seconds(&self) -> Option<u32> {
    match self {
      Self::RateLimitExceeded { .. } => Some(60),
      Self::ServiceUnavailable { .. } => Some(30),
      Self::CircuitBreakerOpen { .. } => Some(120),
      Self::DatabasePoolExhausted => Some(5),
      _ => None,
    }
  }

  // -- Error conversion for clients
  pub fn client_status_and_error(
    &self,
    request_context: &RequestContext,
  ) -> (StatusCode, ClientError) {
    let (status_code, error_code, message, details) = match self {
      // Authentication Errors (401)
      Self::CtxExt(_) => (
        StatusCode::UNAUTHORIZED,
        "AUTHENTICATION_REQUIRED",
        "Authentication required".to_string(),
        None,
      ),
      Self::ApiKeyAuthFailed { reason } => (
        StatusCode::UNAUTHORIZED,
        "API_KEY_INVALID",
        "Invalid API key".to_string(),
        Some(serde_json::json!({ "reason": reason })),
      ),
      Self::JwtValidationFailed { reason } => (
        StatusCode::UNAUTHORIZED,
        "JWT_INVALID",
        "Invalid JWT token".to_string(),
        Some(serde_json::json!({ "reason": reason })),
      ),
      Self::SessionExpired { expired_at } => (
        StatusCode::UNAUTHORIZED,
        "SESSION_EXPIRED",
        "Session has expired".to_string(),
        Some(serde_json::json!({ "expired_at": expired_at })),
      ),

      // Authorization Errors (403)
      Self::InsufficientPermissions { resource } => (
        StatusCode::FORBIDDEN,
        "INSUFFICIENT_PERMISSIONS",
        "Access denied".to_string(),
        Some(serde_json::json!({ "resource": resource })),
      ),

      // Client Errors (400)
      Self::InvalidRequestFormat { message } => {
        (StatusCode::BAD_REQUEST, "INVALID_REQUEST_FORMAT", message.clone(), None)
      }
      Self::MissingRequiredHeader { header } => (
        StatusCode::BAD_REQUEST,
        "MISSING_REQUIRED_HEADER",
        format!("Missing required header: {}", header),
        Some(serde_json::json!({ "header": header })),
      ),
      Self::InvalidHeaderValue { header, value } => (
        StatusCode::BAD_REQUEST,
        "INVALID_HEADER_VALUE",
        format!("Invalid header value: {}", header),
        Some(serde_json::json!({ "header": header, "value": value })),
      ),

      // Not Found (404)
      Self::RouteNotFound { path, method } => (
        StatusCode::NOT_FOUND,
        "ROUTE_NOT_FOUND",
        "Route not found".to_string(),
        Some(serde_json::json!({ "path": path, "method": method })),
      ),

      // Payload Too Large (413)
      Self::RequestTooLarge { size, max_size } => (
        StatusCode::PAYLOAD_TOO_LARGE,
        "REQUEST_TOO_LARGE",
        "Request payload too large".to_string(),
        Some(serde_json::json!({ "size": size, "max_size": max_size })),
      ),

      // Rate Limiting (429)
      Self::RateLimitExceeded { client_id, limit, window } => (
        StatusCode::TOO_MANY_REQUESTS,
        "RATE_LIMIT_EXCEEDED",
        "Rate limit exceeded".to_string(),
        Some(serde_json::json!({
            "client_id": client_id,
            "limit": limit,
            "window": window
        })),
      ),

      // Server Errors (5xx)
      Self::ServiceUnavailable { service } => (
        StatusCode::SERVICE_UNAVAILABLE,
        "SERVICE_UNAVAILABLE",
        "Service temporarily unavailable".to_string(),
        Some(serde_json::json!({ "service": service })),
      ),
      Self::ServiceTimeout { service, timeout_ms } => (
        StatusCode::GATEWAY_TIMEOUT,
        "SERVICE_TIMEOUT",
        "Service request timeout".to_string(),
        Some(serde_json::json!({ "service": service, "timeout_ms": timeout_ms })),
      ),
      Self::Servic { service, status_code, .. } => (
        StatusCode::BAD_GATEWAY,
        "SERVICE_ERROR",
        "Downstream service error".to_string(),
        Some(serde_json::json!({ "service": service, "status_code": status_code })),
      ),
      Self::CircuitBreakerOpen { service } => (
        StatusCode::SERVICE_UNAVAILABLE,
        "CIRCUIT_BREAKER_OPEN",
        "Service circuit breaker is open".to_string(),
        Some(serde_json::json!({ "service": service })),
      ),
      Self::NoHealthyInstances { service } => (
        StatusCode::SERVICE_UNAVAILABLE,
        "NO_HEALTHY_INSTANCES",
        "No healthy service instances available".to_string(),
        Some(serde_json::json!({ "service": service })),
      ),

      // Gateway Internal Errors (500)
      Self::ReqStampNotInReqExt => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "GATEWAY_INTERNAL_ERROR",
        "Gateway internal error".to_string(),
        None,
      ),
      Self::GatewayConfig { config_key } => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "GATEWAY_CONFIG_ERROR",
        "Gateway configuration error".to_string(),
        if cfg!(debug_assertions) {
          Some(serde_json::json!({ "config_key": config_key }))
        } else {
          None
        },
      ),
      Self::RoutingFailed { reason } => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "ROUTING_FAILED",
        "Request routing failed".to_string(),
        if cfg!(debug_assertions) { Some(serde_json::json!({ "reason": reason })) } else { None },
      ),

      // Security Errors
      Self::SuspiciousRequest { reason: _ } => (
        StatusCode::FORBIDDEN,
        "SUSPICIOUS_REQUEST",
        "Request blocked for security reasons".to_string(),
        None, // Never expose security details
      ),
      Self::CorsViolation { origin } => (
        StatusCode::FORBIDDEN,
        "CORS_VIOLATION",
        "CORS policy violation".to_string(),
        Some(serde_json::json!({ "origin": origin })),
      ),
      Self::SecurityPolicyViolation { policy: _ } => (
        StatusCode::FORBIDDEN,
        "SECURITY_POLICY_VIOLATION",
        "Security policy violation".to_string(),
        None, // Don't expose policy details
      ),

      // Other errors default to 500
      _ => (
        StatusCode::INTERNAL_SERVER_ERROR,
        "INTERNAL_SERVER_ERROR",
        "Internal server error".to_string(),
        None,
      ),
    };

    let client_error = ClientError {
      error_code: error_code.to_string(),
      message,
      details,
      timestamp: chrono::Utc::now().to_rfc3339(),
      request_id: request_context.request_id.clone(),
      retry_after: self.retry_after_seconds(),
      trace_id: request_context.trace_id.clone(),
    };

    (status_code, client_error)
  }
}

// ============================================================================
// Request Context
// ============================================================================

#[derive(Debug, Clone)]
pub struct RequestContext {
  pub request_id: Option<String>,
  pub trace_id: Option<String>,
  pub user_id: Option<String>,
  pub client_ip: Option<String>,
  pub user_agent: Option<String>,
  pub start_time: std::time::Instant,
}

impl Default for RequestContext {
  fn default() -> Self {
    Self {
      request_id: None,
      trace_id: None,
      user_id: None,
      client_ip: None,
      user_agent: None,
      start_time: std::time::Instant::now(),
    }
  }
}

// ============================================================================
// Axum Integration
// ============================================================================

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    // Try to extract request context from current task-local storage or create default
    let request_context = extract_request_context_or_default();

    let (status_code, client_error) = self.client_status_and_error(&request_context);

    // Structured logging based on severity and category
    match self.severity() {
      ErrorSeverity::Critical => {
        error!(
            error = %self,
            category = ?self.category(),
            status_code = %status_code,
            request_id = ?request_context.request_id,
            trace_id = ?request_context.trace_id,
            "Critical gateway error"
        );
      }
      ErrorSeverity::High => {
        error!(
            error = %self,
            category = ?self.category(),
            status_code = %status_code,
            request_id = ?request_context.request_id,
            "High severity gateway error"
        );
      }
      ErrorSeverity::Medium => {
        warn!(
            error = %self,
            category = ?self.category(),
            status_code = %status_code,
            request_id = ?request_context.request_id,
            "Medium severity gateway error"
        );
      }
      ErrorSeverity::Low => {
        info!(
            error = %self,
            category = ?self.category(),
            status_code = %status_code,
            request_id = ?request_context.request_id,
            "Client error"
        );
      }
    }

    // Create response with proper headers
    let mut response = (status_code, Json(client_error)).into_response();

    // Add retry-after header for rate limiting
    if let Some(retry_after) = self.retry_after_seconds() {
      response
        .headers_mut()
        .insert("Retry-After", retry_after.to_string().parse().unwrap());
    }

    // Add CORS headers if needed
    if matches!(self, Self::CorsViolation { .. }) {
      response
        .headers_mut()
        .insert("Access-Control-Allow-Origin", "null".parse().unwrap());
    }

    // Insert original error for middleware
    response.extensions_mut().insert(self);
    response.extensions_mut().insert(request_context);

    response
  }
}

// ============================================================================
// Request Context Extraction Utilities
// ============================================================================

/// Extract RequestContext from current task-local storage or create default
/// This works with the middleware that stores context in task-local storage
fn extract_request_context_or_default() -> RequestContext {
  // Try to get from task-local storage first (best option)
  if let Some(context) = CURRENT_REQUEST_CONTEXT
    .try_with(|ctx| ctx.clone())
    .ok()
    .flatten()
  {
    return context;
  }

  // Fallback to default if no context is available
  tracing::warn!("No request context found in task-local storage, using default");
  RequestContext::default()
}

// Task-local storage for request context
tokio::task_local! {
    static CURRENT_REQUEST_CONTEXT: Option<RequestContext>;
}

impl RequestContext {
  /// Create a new RequestContext with generated IDs
  pub fn new() -> Self {
    Self {
      request_id: Some(uuid::Uuid::new_v4().to_string()),
      trace_id: Some(uuid::Uuid::new_v4().to_string()),
      user_id: None,
      client_ip: None,
      user_agent: None,
      start_time: std::time::Instant::now(),
    }
  }

  /// Create RequestContext from HTTP headers
  pub fn from_headers(headers: &axum::http::HeaderMap) -> Self {
    let request_id = headers
      .get("x-request-id")
      .and_then(|h| h.to_str().ok())
      .map(String::from)
      .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let trace_id = headers
      .get("x-trace-id")
      .and_then(|h| h.to_str().ok())
      .map(String::from)
      .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let user_agent = headers
      .get("user-agent")
      .and_then(|h| h.to_str().ok())
      .map(String::from);

    Self {
      request_id: Some(request_id),
      trace_id: Some(trace_id),
      user_id: None,   // Will be set by auth middleware
      client_ip: None, // Will be set by IP extraction middleware
      user_agent,
      start_time: std::time::Instant::now(),
    }
  }

  /// Set the current request context in task-local storage
  pub async fn run_with_context<F, R>(self, future: F) -> R
  where
    F: std::future::Future<Output = R>,
  {
    CURRENT_REQUEST_CONTEXT.scope(Some(self), future).await
  }

  /// Update user information (called by auth middleware)
  pub fn with_user_id(mut self, user_id: String) -> Self {
    self.user_id = Some(user_id);
    self
  }

  /// Update client IP (called by IP extraction middleware)
  pub fn with_client_ip(mut self, client_ip: String) -> Self {
    self.client_ip = Some(client_ip);
    self
  }
}

// ============================================================================
// Convenience From Implementations
// ============================================================================

impl From<reqwest::Error> for Error {
  fn from(err: reqwest::Error) -> Self {
    if err.is_timeout() {
      Self::service_timeout("unknown", 30000) // Default timeout
    } else if err.is_connect() {
      Self::service_unavailable("unknown")
    } else {
      Self::service_error("unknown", 500, Some(err.to_string()))
    }
  }
}

impl From<redis::RedisError> for Error {
  fn from(err: redis::RedisError) -> Self {
    Self::RedisConnectionFailed { source: Box::new(err) }
  }
}
