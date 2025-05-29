use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Sui client error: {0}")]
  SuiClient(String),

  #[error("Invalid request: {0}")]
  InvalidRequest(String),

  #[error("Internal error: {0}")]
  Internal(String),
}
