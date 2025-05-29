use crate::Result;
use async_trait::async_trait;
use sui_sdk::rpc_types::Coin;

#[async_trait]
pub trait SuiRepository: Send + Sync {
  // Read operations
  async fn fetch_coin() -> Result<Option<Coin>>;
}
