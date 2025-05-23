mod error;
mod log;
pub mod middleware;
pub mod routes;

pub type Result<T> = std::result::Result<T, error::Error>;
