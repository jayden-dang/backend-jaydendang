use std::fmt::Display;

use derive_more::From;
use jd_storage::dbx;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = std::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
    CantCreateModelManagerProvider(String),

    #[from]
    Dbx(dbx::Error),

    #[from]
    SeaQuery(#[serde_as(as = "DisplayFromStr")] sea_query::error::Error),

    #[from]
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::error::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}
