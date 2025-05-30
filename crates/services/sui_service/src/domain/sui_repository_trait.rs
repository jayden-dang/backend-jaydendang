use crate::Result;
use crate::models::{GasPoolStatus, UserStats};
use async_trait::async_trait;
use sui_sdk::rpc_types::{
  Coin, SuiObjectResponse, SuiTransactionBlockResponse, SuiEvent, Page,
  Balance, SuiCoinMetadata, SuiObjectDataOptions,
  SuiTransactionBlockResponseOptions, DynamicFieldInfo
};
use sui_sdk::types::base_types::{SuiAddress, TransactionDigest};
use sui_types::{base_types::ObjectID, transaction::Transaction};

#[async_trait]
pub trait SuiRepository: Send + Sync {
  // ============== COIN OPERATIONS ==============
  /// Get coins for a specific address and coin type
  async fn get_coins(
    &self,
    address: SuiAddress,
    coin_type: Option<String>,
    cursor: Option<String>,
    limit: Option<usize>,
  ) -> Result<Page<Coin, String>>;

  /// Get all coins for a specific address
  async fn get_all_coins(
    &self,
    address: SuiAddress,
    cursor: Option<String>,
    limit: Option<usize>,
  ) -> Result<Page<Coin, String>>;

  /// Get balance for a specific coin type
  async fn get_balance(&self, address: SuiAddress, coin_type: Option<String>) -> Result<Balance>;

  /// Get all balances for an address
  async fn get_all_balances(&self, address: SuiAddress) -> Result<Vec<Balance>>;

  /// Get coin metadata
  async fn get_coin_metadata(&self, coin_type: String) -> Result<Option<SuiCoinMetadata>>;

  /// Get total supply for a coin type
  async fn get_total_supply(&self, coin_type: String) -> Result<Option<u64>>;

  /// Select coins for a transaction
  async fn select_coins(
    &self,
    address: SuiAddress,
    coin_type: Option<String>,
    amount: u64,
    exclude: Vec<ObjectID>,
  ) -> Result<Vec<Coin>>;

  // ============== OBJECT OPERATIONS ==============
  /// Get object details
  async fn get_object(
    &self,
    object_id: ObjectID,
    options: Option<SuiObjectDataOptions>,
  ) -> Result<SuiObjectResponse>;

  /// Get multiple objects
  async fn get_objects(
    &self,
    object_ids: Vec<ObjectID>,
    options: Option<SuiObjectDataOptions>,
  ) -> Result<Vec<SuiObjectResponse>>;

  /// Get owned objects by address
  async fn get_owned_objects(
    &self,
    address: SuiAddress,
    query: Option<String>,
    cursor: Option<ObjectID>,
    limit: Option<usize>,
  ) -> Result<Page<SuiObjectResponse, ObjectID>>;

  /// Get dynamic fields
  async fn get_dynamic_fields(
    &self,
    parent_object_id: ObjectID,
    cursor: Option<ObjectID>,
    limit: Option<usize>,
  ) -> Result<Page<DynamicFieldInfo, ObjectID>>;

  // ============== TRANSACTION OPERATIONS ==============
  /// Get transaction block
  async fn get_transaction_block(
    &self,
    digest: TransactionDigest,
    options: Option<SuiTransactionBlockResponseOptions>,
  ) -> Result<SuiTransactionBlockResponse>;

  /// Get multiple transaction blocks
  async fn get_transaction_blocks(
    &self,
    digests: Vec<TransactionDigest>,
    options: Option<SuiTransactionBlockResponseOptions>,
  ) -> Result<Vec<SuiTransactionBlockResponse>>;

  // ============== EVENT OPERATIONS ==============
  /// Get events for a transaction
  async fn get_events(&self, digest: TransactionDigest) -> Result<Vec<SuiEvent>>;


  // ============== NETWORK INFO ==============
  /// Get latest checkpoint sequence number
  async fn get_latest_checkpoint_sequence_number(&self) -> Result<u64>;

  /// Get total transaction blocks
  async fn get_total_transaction_blocks(&self) -> Result<u64>;

  /// Get reference gas price
  async fn get_reference_gas_price(&self) -> Result<u64>;

  /// Get chain identifier
  async fn get_chain_identifier(&self) -> Result<String>;

  // ============== LEGACY/DEPRECATED - Keep for backward compatibility ==============
  async fn fetch_coin(&self, sender: String) -> Result<Option<Coin>>;

  // ============== GAS STATION OPERATIONS ==============
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
