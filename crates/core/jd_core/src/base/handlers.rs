use crate::AppState;
use crate::ctx::Ctx;
use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use validator::Validate;

/// Base handler trait for standardizing service handlers
#[async_trait]
pub trait HandlerBase {
  type Request: DeserializeOwned + Validate + Send + Sync + Clone;
  type Response: Serialize + Send + Sync;
  type Error: super::error::ServiceError;

  /// Get the handler name for logging
  fn handler_name(&self) -> &'static str;

  /// Validate the request
  async fn validate_request(&self, request: &Self::Request) -> Result<(), Self::Error> {
    request
      .validate()
      .map_err(|e| Self::map_validation_error(e))?;
    Ok(())
  }

  /// Map validation errors to service errors
  fn map_validation_error(err: validator::ValidationErrors) -> Self::Error;

  /// Pre-processing hook (e.g., authorization checks)
  async fn pre_process(&self, _ctx: &Ctx, _request: &Self::Request) -> Result<(), Self::Error> {
    Ok(())
  }

  /// Main handler logic
  async fn handle(&self, ctx: Ctx, request: Self::Request) -> Result<Self::Response, Self::Error>;

  /// Post-processing hook (e.g., audit logging)
  async fn post_process(
    &self,
    _ctx: &Ctx,
    _request: &Self::Request,
    _response: &Self::Response,
  ) -> Result<(), Self::Error> {
    Ok(())
  }

  /// Execute the handler with all hooks
  async fn execute(&self, ctx: Ctx, request: Self::Request) -> Result<Self::Response, Self::Error> {
    // Validate request
    self.validate_request(&request).await?;

    // Pre-process
    self.pre_process(&ctx, &request).await?;

    // Handle
    let response = self.handle(ctx.clone(), request.clone()).await?;

    // Post-process
    self.post_process(&ctx, &request, &response).await?;

    Ok(response)
  }
}

/// Handler with state management
#[async_trait]
pub trait StatefulHandler: HandlerBase {
  /// Get the application state
  fn state(&self) -> Arc<AppState>;
}

/// Handler with repository access
#[async_trait]
pub trait RepositoryHandler<R>: StatefulHandler
where
  R: Send + Sync,
{
  /// Create a repository instance
  fn repository(&self) -> R;
}

/// Macro to implement common handler functionality
#[macro_export]
macro_rules! impl_handler {
  ($handler:ty, $request:ty, $response:ty, $error:ty, $name:literal) => {
    #[async_trait::async_trait]
    impl HandlerBase for $handler {
      type Request = $request;
      type Response = $response;
      type Error = $error;

      fn handler_name(&self) -> &'static str {
        $name
      }

      fn map_validation_error(err: validator::ValidationErrors) -> Self::Error {
        use $crate::service_error::ValidationError;
        Self::Error::from(ValidationError { details: err.into() })
      }
    }
  };
}

/// Macro to implement stateful handler
#[macro_export]
macro_rules! impl_stateful_handler {
  ($handler:ty) => {
    #[async_trait::async_trait]
    impl StatefulHandler for $handler {
      fn state(&self) -> std::sync::Arc<$crate::AppState> {
        self.state.clone()
      }
    }
  };
}

/// Standard handler response wrapper
#[derive(Debug, serde::Serialize)]
pub struct HandlerResponse<T> {
  pub success: bool,
  pub data: Option<T>,
  pub message: Option<String>,
  pub timestamp: String,
}

impl<T: serde::Serialize> HandlerResponse<T> {
  pub fn success(data: T) -> Self {
    Self {
      success: true,
      data: Some(data),
      message: None,
      timestamp: chrono::Utc::now().to_rfc3339(),
    }
  }

  pub fn error(message: String) -> Self {
    Self {
      success: false,
      data: None,
      message: Some(message),
      timestamp: chrono::Utc::now().to_rfc3339(),
    }
  }
}
