use std::fmt::Display;

use axum::response::{IntoResponse, Response};
use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, Clone, From)]
#[serde(tag = "type", content = "data")]
pub enum Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
