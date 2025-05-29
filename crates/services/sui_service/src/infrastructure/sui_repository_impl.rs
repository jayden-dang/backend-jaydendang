use crate::{domain::sui_repository_trait::SuiRepository, Result};
use async_trait::async_trait;
use jd_core::sui::sui_client::SuiClient;
use sui_sdk::rpc_types::Coin;

pub struct SuiRepositoryImpl {
  pub client: SuiClient,
}

impl SuiRepositoryImpl {
  pub fn new(client: SuiClient) -> Self {
    Self { client }
  }
}

#[async_trait]
impl SuiRepository for SuiRepositoryImpl {
  async fn fetch_coin() -> Result<Option<Coin>> {
    todo!()
  }
}
