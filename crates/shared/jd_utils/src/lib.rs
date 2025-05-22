pub mod config;
mod error;

pub type Result<T> = std::result::Result<T, error::Error>;
