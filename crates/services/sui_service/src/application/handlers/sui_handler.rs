use crate::{
    application::use_cases::sui_use_cases::SuiUseCases,
    domain::sui_repository_trait::SuiRepository,
    Result,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsRequest {
    pub event_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetObjectRequest {
    pub object_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetBalanceRequest {
    pub address: String,
}

pub struct SuiHandler<R: SuiRepository> {
    use_cases: SuiUseCases<R>,
}

impl<R: SuiRepository> SuiHandler<R> {
    pub fn new(use_cases: SuiUseCases<R>) -> Self {
        Self { use_cases }
    }

    pub async fn get_api_version(&self) -> Result<Json<String>> {
        let version = self.use_cases.get_api_version().await?;
        Ok(Json(version))
    }

    pub async fn get_events(&self, Json(req): Json<GetEventsRequest>) -> Result<Json<Vec<String>>> {
        let events = self.use_cases.get_events(req.event_type).await?;
        Ok(Json(events))
    }

    pub async fn get_object(&self, Json(req): Json<GetObjectRequest>) -> Result<Json<String>> {
        let object = self.use_cases.get_object(req.object_id).await?;
        Ok(Json(object))
    }

    pub async fn get_balance(&self, Json(req): Json<GetBalanceRequest>) -> Result<Json<String>> {
        let balance = self.use_cases.get_balance(req.address).await?;
        Ok(Json(balance))
    }
} 