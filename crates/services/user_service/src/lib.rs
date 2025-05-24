mod base;
mod error;
pub mod infra;

pub type Result<T> = std::result::Result<T, error::Error>;
