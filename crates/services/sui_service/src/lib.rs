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
    read_service: application::read_service::ReadService,
    event_service: application::event_service::EventService,
}

impl SuiService {
    pub async fn new(client: SuiClient) -> Self {
        let read_service = application::read_service::ReadService::new(client.clone());
        let event_service = application::event_service::EventService::new(client.clone());
        
        Self { 
            client,
            read_service,
            event_service,
        }
    }

    pub async fn get_api_version(&self) -> Result<String> {
        self.client.get_api_version().await
    }

    pub fn read_service(&self) -> &application::read_service::ReadService {
        &self.read_service
    }

    pub fn event_service(&self) -> &application::event_service::EventService {
        &self.event_service
    }
}

