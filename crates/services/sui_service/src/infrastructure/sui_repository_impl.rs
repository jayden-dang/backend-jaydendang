use crate::{domain::sui_repository_trait::SuiRepository, error::Error, Result};
use async_trait::async_trait;
use futures::{future, StreamExt};
use jd_core::AppState;
use std::str::FromStr;
use sui_sdk::{rpc_types::Coin, types::base_types::SuiAddress};

#[derive(Clone)]
pub struct SuiRepositoryImpl {
  app_state: AppState,
}

impl SuiRepositoryImpl {
  pub fn new(app_state: AppState) -> Self {
    Self { app_state }
  }
}

#[async_trait]
impl SuiRepository for SuiRepositoryImpl {
  async fn fetch_coin(&self, sender: String) -> Result<Option<Coin>> {
    let coin_type = "0x2::sui::SUI".to_string();
    let address = SuiAddress::from_str(&sender).map_err(|e| Error::InvalidRequest(e.to_string()))?;
    let coins_stream = self
      .app_state
      .sui_client
      .client
      .coin_read_api()
      .get_coins_stream(address, Some(coin_type));

    let mut coins = coins_stream
      .skip_while(|c| future::ready(c.balance < 5_000_000))
      .boxed();

    Ok(coins.next().await)
  }
}
