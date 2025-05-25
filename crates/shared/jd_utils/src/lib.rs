pub mod config;
mod error;
pub mod macros;
pub mod time;
pub mod regex;

pub use macros::*;

pub type Result<T> = std::result::Result<T, error::Error>;
