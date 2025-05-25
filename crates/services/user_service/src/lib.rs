mod error;
pub mod users;

pub type Result<T> = std::result::Result<T, error::Error>;
