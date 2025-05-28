use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, thiserror::Error)]
pub enum Error {
  #[error("Cannot Create new Root Context: {message}")]
  CtxCannotNewRootCtx { message: String },
}
