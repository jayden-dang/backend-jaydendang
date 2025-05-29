use crate::Result;
use async_trait::async_trait;
use sui_sdk::{rpc_types::Coin, types::base_types::SuiAddress};

#[async_trait]
pub trait SuiRepository: Send + Sync {
  // Read operations
  async fn fetch_coin(&self, sender: String) -> Result<Option<Coin>>;
}
