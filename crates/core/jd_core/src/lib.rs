pub mod config;
mod error;
mod store;

pub type Result<T> = std::result::Result<T, error::Error>;

pub struct ModelManager {}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        Ok(ModelManager {})
    }
}
