use std::fmt::Display;

use derive_more::From;
use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};
use validator::ValidationErrors;

#[serde_as]
#[derive(Debug, From, Serialize)]
pub enum Error {
  #[from]
  ValidationErrors(#[serde_as(as = "DisplayFromStr")] ValidationErrors),
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

impl std::error::Error for Error {}
