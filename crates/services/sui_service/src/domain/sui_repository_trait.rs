use crate::Result;
use async_trait::async_trait;
use jd_core::sui::SuiClient;

#[async_trait]
pub trait SuiRepository: Send + Sync {
    async fn get_api_version(&self) -> Result<String>;
    async fn get_events(&self, event_type: String) -> Result<Vec<String>>;
    async fn get_object(&self, object_id: String) -> Result<String>;
    async fn get_balance(&self, address: String) -> Result<String>;
} 