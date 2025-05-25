pub mod config;
mod error;
pub mod macros;
pub mod regex;
pub mod time;

pub use macros::*;

pub type Result<T> = std::result::Result<T, error::Error>;
