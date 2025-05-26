mod error;
pub mod users;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;
