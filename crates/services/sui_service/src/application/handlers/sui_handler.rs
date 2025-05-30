use crate::Result;
use crate::infrastructure::enhanced_sui_repository::EnhancedSuiRepository;
use axum::{
  Json,
  extract::State,
};
use jd_core::AppState;
use serde_json::{Value, json};
use sui_sdk::rpc_types::Coin;

use crate::{
  application::use_cases::sui_use_cases::SuiUseCases, 
  domain::sui_repository_trait::SuiRepository,
};

pub struct SuiHandler<R: SuiRepository> {
  pub use_cases: SuiUseCases<R>,
}

impl<R: SuiRepository> SuiHandler<R> {
  pub fn new(use_cases: SuiUseCases<R>) -> Self {
    Self { use_cases }
  }

  pub async fn fetch_coin(
    State(state): State<AppState>,
    Json(req): Json<String>,
  ) -> Result<Json<Coin>> {
    let repository = EnhancedSuiRepository::new(state);
    let use_cases = SuiUseCases::new(repository);
    let object = use_cases.fetch_coin(req).await?;
    Ok(Json(object))
  }

  // Simple health check
  pub async fn health_check() -> &'static str {
    "Sui Service is healthy"
  }

  // Simple endpoint for testing
  pub async fn test_connection(State(state): State<AppState>) -> Result<Json<Value>> {
    let repository = EnhancedSuiRepository::new(state);
    
    match repository.get_latest_checkpoint_sequence_number().await {
      Ok(checkpoint) => Ok(Json(json!({
        "status": "success",
        "latest_checkpoint": checkpoint,
        "message": "Successfully connected to Sui network"
      }))),
      Err(e) => Ok(Json(json!({
        "status": "error",
        "message": format!("Failed to connect to Sui network: {}", e)
      })))
    }
  }
}