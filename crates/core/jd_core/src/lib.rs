pub mod ctx;
mod error;

pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(Clone)]
pub struct ModelManager {}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        Ok(ModelManager {})
    }
}
