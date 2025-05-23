mod error;
mod log;
pub mod mw;

pub type Result<T> = std::result::Result<T, error::Error>;
