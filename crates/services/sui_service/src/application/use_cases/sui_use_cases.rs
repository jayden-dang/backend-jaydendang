use crate::error::Error;
use crate::Result;
use sui_sdk::rpc_types::Coin;
use sui_sdk::types::base_types::SuiAddress;

use crate::domain::sui_repository_trait::SuiRepository;

#[derive(Clone)]
pub struct SuiUseCases<R: SuiRepository> {
  pub repository: R,
}

impl<R: SuiRepository> SuiUseCases<R> {
  pub fn new(repository: R) -> Self {
    Self { repository }
  }

  pub async fn fetch_coin(&self, sender: String) -> Result<Coin> {
    self
      .repository
      .fetch_coin(sender)
      .await?
      .ok_or_else(|| Error::InvalidRequest("Coin not found".into()))
  }
}
