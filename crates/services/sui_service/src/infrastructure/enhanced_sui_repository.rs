use crate::domain::sui_repository_trait::SuiRepository;
use crate::{Result, error::Error};
use async_trait::async_trait;
use jd_core::AppState;
use sui_sdk::rpc_types::{
  Coin, SuiObjectResponse, SuiTransactionBlockResponse, SuiEvent, Page,
  Balance, SuiCoinMetadata, SuiObjectDataOptions,
  SuiTransactionBlockResponseOptions, DynamicFieldInfo
};
use sui_sdk::types::base_types::{SuiAddress, TransactionDigest};
use sui_types::base_types::ObjectID;

/// Enhanced Sui repository implementation with full SDK integration
/// This implementation provides comprehensive access to Sui blockchain data
#[derive(Clone)]
pub struct EnhancedSuiRepository {
  app_state: AppState,
}

impl EnhancedSuiRepository {
  pub fn new(app_state: AppState) -> Self {
    Self { app_state }
  }
}

#[async_trait]
impl SuiRepository for EnhancedSuiRepository {
  // ============== COIN OPERATIONS ==============

  async fn get_coins(
    &self,
    address: SuiAddress,
    coin_type: Option<String>,
    cursor: Option<String>,
    limit: Option<usize>,
  ) -> Result<Page<Coin, String>> {
    self.app_state
      .sui_client.client
      .coin_read_api()
      .get_coins(address, coin_type, cursor, limit)
      .await
      .map_err(|e| Error::Internal(format!("Failed to get coins: {}", e)))
  }

  async fn get_all_coins(
    &self,
    address: SuiAddress,
    cursor: Option<String>,
    limit: Option<usize>,
  ) -> Result<Page<Coin, String>> {
    self.app_state
      .sui_client.client
      .coin_read_api()
      .get_all_coins(address, cursor, limit)
      .await
      .map_err(|e| Error::Internal(format!("Failed to get all coins: {}", e)))
  }

  async fn get_balance(&self, address: SuiAddress, coin_type: Option<String>) -> Result<Balance> {
    self.app_state
      .sui_client.client
      .coin_read_api()
      .get_balance(address, coin_type)
      .await
      .map_err(|e| Error::Internal(format!("Failed to get balance: {}", e)))
  }

  async fn get_all_balances(&self, address: SuiAddress) -> Result<Vec<Balance>> {
    self.app_state
      .sui_client.client
      .coin_read_api()
      .get_all_balances(address)
      .await
      .map_err(|e| Error::Internal(format!("Failed to get all balances: {}", e)))
  }

  async fn get_coin_metadata(&self, coin_type: String) -> Result<Option<SuiCoinMetadata>> {
    self.app_state
      .sui_client.client
      .coin_read_api()
      .get_coin_metadata(coin_type)
      .await
      .map_err(|e| Error::Internal(format!("Failed to get coin metadata: {}", e)))
  }

  async fn get_total_supply(&self, coin_type: String) -> Result<Option<u64>> {
    let supply = self.app_state
      .sui_client.client
      .coin_read_api()
      .get_total_supply(coin_type)
      .await
      .map_err(|e| Error::Internal(format!("Failed to get total supply: {}", e)))?;
    Ok(Some(supply.value))
  }

  async fn select_coins(
    &self,
    address: SuiAddress,
    coin_type: Option<String>,
    amount: u64,
    exclude: Vec<ObjectID>,
  ) -> Result<Vec<Coin>> {
    self.app_state
      .sui_client.client
      .coin_read_api()
      .select_coins(address, coin_type, amount.into(), exclude)
      .await
      .map_err(|e| Error::Internal(format!("Failed to select coins: {}", e)))
  }

  // ============== OBJECT OPERATIONS ==============

  async fn get_object(
    &self,
    object_id: ObjectID,
    options: Option<SuiObjectDataOptions>,
  ) -> Result<SuiObjectResponse> {
    self.app_state
      .sui_client.client
      .read_api()
      .get_object_with_options(object_id, options.unwrap_or_default())
      .await
      .map_err(|e| Error::Internal(format!("Failed to get object: {}", e)))
  }

  async fn get_objects(
    &self,
    object_ids: Vec<ObjectID>,
    options: Option<SuiObjectDataOptions>,
  ) -> Result<Vec<SuiObjectResponse>> {
    self.app_state
      .sui_client.client
      .read_api()
      .multi_get_object_with_options(object_ids, options.unwrap_or_default())
      .await
      .map_err(|e| Error::Internal(format!("Failed to get objects: {}", e)))
  }

  async fn get_owned_objects(
    &self,
    address: SuiAddress,
    query: Option<String>,
    cursor: Option<ObjectID>,
    limit: Option<usize>,
  ) -> Result<Page<SuiObjectResponse, ObjectID>> {
    // Convert string query to proper query type if needed
    let query = query.map(|_| sui_sdk::rpc_types::SuiObjectResponseQuery::new_with_options(
      SuiObjectDataOptions::full_content(),
    ));
    
    self.app_state
      .sui_client.client
      .read_api()
      .get_owned_objects(address, query, cursor, limit)
      .await
      .map_err(|e| Error::Internal(format!("Failed to get owned objects: {}", e)))
  }

  async fn get_dynamic_fields(
    &self,
    parent_object_id: ObjectID,
    cursor: Option<ObjectID>,
    limit: Option<usize>,
  ) -> Result<Page<DynamicFieldInfo, ObjectID>> {
    self.app_state
      .sui_client.client
      .read_api()
      .get_dynamic_fields(parent_object_id, cursor, limit)
      .await
      .map_err(|e| Error::Internal(format!("Failed to get dynamic fields: {}", e)))
  }

  // ============== TRANSACTION OPERATIONS ==============

  async fn get_transaction_block(
    &self,
    digest: TransactionDigest,
    options: Option<SuiTransactionBlockResponseOptions>,
  ) -> Result<SuiTransactionBlockResponse> {
    self.app_state
      .sui_client.client
      .read_api()
      .get_transaction_with_options(digest, options.unwrap_or_default())
      .await
      .map_err(|e| Error::Internal(format!("Failed to get transaction: {}", e)))
  }

  async fn get_transaction_blocks(
    &self,
    digests: Vec<TransactionDigest>,
    options: Option<SuiTransactionBlockResponseOptions>,
  ) -> Result<Vec<SuiTransactionBlockResponse>> {
    self.app_state
      .sui_client.client
      .read_api()
      .multi_get_transactions_with_options(digests, options.unwrap_or_default())
      .await
      .map_err(|e| Error::Internal(format!("Failed to get transactions: {}", e)))
  }

  // ============== EVENT OPERATIONS ==============

  async fn get_events(&self, digest: TransactionDigest) -> Result<Vec<SuiEvent>> {
    self.app_state
      .sui_client.client
      .event_api()
      .get_events(digest)
      .await
      .map_err(|e| Error::Internal(format!("Failed to get events: {}", e)))
  }


  // ============== NETWORK INFO ==============

  async fn get_latest_checkpoint_sequence_number(&self) -> Result<u64> {
    self.app_state
      .sui_client.client
      .read_api()
      .get_latest_checkpoint_sequence_number()
      .await
      .map_err(|e| Error::Internal(format!("Failed to get latest checkpoint: {}", e)))
  }

  async fn get_total_transaction_blocks(&self) -> Result<u64> {
    self.app_state
      .sui_client.client
      .read_api()
      .get_total_transaction_blocks()
      .await
      .map_err(|e| Error::Internal(format!("Failed to get total transactions: {}", e)))
  }

  async fn get_reference_gas_price(&self) -> Result<u64> {
    self.app_state
      .sui_client.client
      .governance_api()
      .get_reference_gas_price()
      .await
      .map_err(|e| Error::Internal(format!("Failed to get gas price: {}", e)))
  }

  async fn get_chain_identifier(&self) -> Result<String> {
    self.app_state
      .sui_client.client
      .read_api()
      .get_chain_identifier()
      .await
      .map_err(|e| Error::Internal(format!("Failed to get chain identifier: {}", e)))
  }

  // ============== LEGACY/DEPRECATED - Keep for backward compatibility ==============

  async fn fetch_coin(&self, sender: String) -> Result<Option<Coin>> {
    use std::str::FromStr;
    
    let address = SuiAddress::from_str(&sender)
      .map_err(|_| Error::InvalidRequest("Invalid address format".to_string()))?;
      
    let coins = self.get_coins(address, None, None, Some(1)).await?;
    Ok(coins.data.into_iter().next())
  }

  // ============== PLACEHOLDER IMPLEMENTATIONS ==============
  // These methods need to be implemented based on your specific gas station logic

  async fn get_available_gas(&self, _required_budget: u64) -> Result<ObjectID> {
    Err(Error::ImplementationPending("Gas station operations not implemented in enhanced repository".to_string()))
  }

  async fn release_gas(&self, _object_id: ObjectID) -> Result<()> {
    Err(Error::ImplementationPending("Gas station operations not implemented in enhanced repository".to_string()))
  }

  async fn sponsor_transaction(
    &self,
    _tx_bytes: Vec<u8>,
    _user_signature: &[u8],
  ) -> Result<(sui_types::transaction::Transaction, String)> {
    Err(Error::ImplementationPending("Sponsored transactions not implemented in enhanced repository".to_string()))
  }

  async fn get_pool_stats(&self) -> Result<crate::models::GasPoolStatus> {
    Err(Error::ImplementationPending("Gas pool stats not implemented in enhanced repository".to_string()))
  }

  async fn refresh_gas_pool(&self) -> Result<()> {
    Err(Error::ImplementationPending("Gas pool refresh not implemented in enhanced repository".to_string()))
  }

  async fn log_sponsored_transaction(
    &self,
    _user_address: &SuiAddress,
    _gas_budget: u64,
  ) -> Result<()> {
    Err(Error::ImplementationPending("Transaction logging not implemented in enhanced repository".to_string()))
  }

  async fn get_user_stats(&self, _address: &str) -> Result<Option<crate::models::UserStats>> {
    Err(Error::ImplementationPending("User stats not implemented in enhanced repository".to_string()))
  }

  async fn check_rate_limit(&self, _user_address: &SuiAddress) -> Result<bool> {
    Ok(true) // Always allow for enhanced repository
  }
}