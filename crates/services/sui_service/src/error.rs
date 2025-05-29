use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Sui client error: {0}")]
  SuiClient(String),

  #[error("Invalid request: {0}")]
  InvalidRequest(String),

  #[error("Internal error: {0}")]
  Internal(String),

  #[error("Implementation pending: {0}")]
  ImplementationPending(String),
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    let (status, error_message) = match self {
      Error::SuiClient(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
      Error::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
      Error::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
      Error::ImplementationPending(msg) => (StatusCode::OK, msg),
    };

    (status, error_message).into_response()
  }
}
