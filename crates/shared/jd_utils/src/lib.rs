pub mod config;
mod error;
pub mod time;

pub type Result<T> = std::result::Result<T, error::Error>;
