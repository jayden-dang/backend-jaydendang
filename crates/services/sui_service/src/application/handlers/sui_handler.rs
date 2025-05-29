use crate::infrastructure::sui_repository_impl::SuiRepositoryImpl;
use crate::Result;
use axum::{extract::State, Json};
use jd_core::AppState;
use sui_sdk::rpc_types::Coin;

use crate::{
  application::use_cases::sui_use_cases::SuiUseCases, domain::sui_repository_trait::SuiRepository,
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
    let repository = SuiRepositoryImpl::new(state);
    let use_cases = SuiUseCases::new(repository);
    let object = use_cases.fetch_coin(req).await?;
    Ok(Json(object))
  }
}
