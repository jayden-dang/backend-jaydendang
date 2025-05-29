// -->>> Region:: START  --->>>  Public Modules
pub mod application;
pub mod infrastructure;
pub mod models;
// <<<-- Region:: END    <<<---  Public Modules

mod domain;
mod error;

use error::Error;
type Result<T> = std::result::Result<T, Error>;

use jd_core::sui::SuiClient;

pub struct SuiService {
    client: SuiClient,
}

impl SuiService {
    pub async fn new(client: SuiClient) -> Self {
        Self { client }
    }

    pub async fn get_api_version(&self) -> Result<String> {
        self.client.get_api_version().await
    }
}
