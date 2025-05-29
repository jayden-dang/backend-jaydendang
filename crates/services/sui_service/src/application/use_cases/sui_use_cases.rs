use crate::{
    domain::sui_repository_trait::SuiRepository,
    Result,
};

pub struct SuiUseCases<R: SuiRepository> {
    repository: R,
}

impl<R: SuiRepository> SuiUseCases<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_api_version(&self) -> Result<String> {
        self.repository.get_api_version().await
    }

    pub async fn get_events(&self, event_type: String) -> Result<Vec<String>> {
        self.repository.get_events(event_type).await
    }

    pub async fn get_object(&self, object_id: String) -> Result<String> {
        self.repository.get_object(object_id).await
    }

    pub async fn get_balance(&self, address: String) -> Result<String> {
        self.repository.get_balance(address).await
    }
} 