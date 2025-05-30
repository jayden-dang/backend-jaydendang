use crate::Result;
use crate::models::{GasPoolStatus, UserStats};
use async_trait::async_trait;
use sui_sdk::{rpc_types::Coin, types::base_types::SuiAddress};
use sui_types::{base_types::ObjectID, transaction::Transaction};

#[async_trait]
pub trait SuiRepository: Send + Sync {
  // Read operations
  async fn fetch_coin(&self, sender: String) -> Result<Option<Coin>>;

  // Gas Station operations
  async fn get_available_gas(&self, required_budget: u64) -> Result<ObjectID>;
  async fn release_gas(&self, object_id: ObjectID) -> Result<()>;
  async fn sponsor_transaction(
    &self,
    tx_bytes: Vec<u8>,
    user_signature: &[u8],
  ) -> Result<(Transaction, String)>;
  async fn get_pool_stats(&self) -> Result<GasPoolStatus>;
  async fn refresh_gas_pool(&self) -> Result<()>;
  async fn log_sponsored_transaction(
    &self,
    user_address: &SuiAddress,
    gas_budget: u64,
  ) -> Result<()>;
  async fn get_user_stats(&self, address: &str) -> Result<Option<UserStats>>;
  async fn check_rate_limit(&self, user_address: &SuiAddress) -> Result<bool>;
}
