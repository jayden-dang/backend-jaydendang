use std::fmt::Display;

use derive_more::From;
use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
  #[from]
  Config(#[serde_as(as = "DisplayFromStr")] config::ConfigError),
}

impl Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

impl std::error::Error for Error {}
