use crate::infrastructure::gas_station::GasStation;
use crate::models::{GasPoolStatus, UserStats};
use crate::{Result, domain::sui_repository_trait::SuiRepository, error::Error};
use async_trait::async_trait;
use blake2::{Blake2b, Digest};
use fastcrypto::{
  ed25519::Ed25519KeyPair,
  traits::{KeyPair, ToFromBytes},
};
use futures::{StreamExt, future};
use jd_core::AppState;
use jd_utils::time;
use redis::AsyncCommands;
use std::str::FromStr;
use std::sync::Arc;
use sui_keys::keystore::{AccountKeystore, InMemKeystore};
use sui_sdk::{rpc_types::Coin, types::base_types::SuiAddress};
use sui_types::crypto::SuiKeyPair;
use sui_types::{
  base_types::ObjectID,
  crypto::Signer,
  transaction::{Transaction, TransactionData},
};

#[derive(Clone)]
pub struct SuiRepositoryImpl {
  app_state: AppState,
  gas_station: Option<Arc<GasStation>>,
  sponsor_keystore: Option<Arc<InMemKeystore>>,
}

impl SuiRepositoryImpl {
  pub fn new(app_state: AppState) -> Self {
    Self { app_state, gas_station: None, sponsor_keystore: None }
  }

  pub async fn with_gas_station(
    app_state: AppState,
    sui_rpc_url: &str,
    sponsor_address: SuiAddress,
    max_gas_budget: u64,
  ) -> Result<Self> {
    let gas_station = GasStation::new(sui_rpc_url, sponsor_address, max_gas_budget)
      .await
      .map_err(|e| Error::Internal(e.to_string()))?;

    Ok(Self { app_state, gas_station: Some(Arc::new(gas_station)), sponsor_keystore: None })
  }

  pub async fn with_gas_station_and_key(
    app_state: AppState,
    sui_rpc_url: &str,
    sponsor_address: SuiAddress,
    sponsor_private_key: &str,
    max_gas_budget: u64,
  ) -> Result<Self> {
    let gas_station = GasStation::new(sui_rpc_url, sponsor_address, max_gas_budget)
      .await
      .map_err(|e| Error::Internal(e.to_string()))?;

    // Create keystore with sponsor private key
    let mut keystore = InMemKeystore::default();

    // Parse private key (handle both with and without 0x prefix)
    let key_str = sponsor_private_key
      .strip_prefix("0x")
      .unwrap_or(sponsor_private_key);

    let key_bytes = hex::decode(key_str)
      .map_err(|e| Error::Internal(format!("Invalid private key format: {}", e)))?;

    if key_bytes.len() != 32 {
      return Err(Error::Internal("Private key must be 32 bytes".to_string()));
    }

    let keypair = Ed25519KeyPair::from_bytes(&key_bytes)
      .map_err(|e| Error::Internal(format!("Failed to create keypair: {}", e)))?;

    // Add key to keystore manually since import_from_keypair doesn't exist
    // Sui addresses are derived by hashing [scheme_flag || public_key_bytes] with BLAKE2b
    let scheme_flag = 0u8; // Ed25519 flag is 0
    let public_key_bytes = keypair.public().as_bytes();
    let mut hasher_input = Vec::new();
    hasher_input.push(scheme_flag);
    hasher_input.extend_from_slice(public_key_bytes);

    let mut hasher = Blake2b::<blake2::digest::consts::U32>::new();
    hasher.update(&hasher_input);
    let hash_result = hasher.finalize();
    let sui_address = SuiAddress::from_bytes(hash_result.as_slice())
      .map_err(|e| Error::Internal(format!("Failed to create SuiAddress: {}", e)))?;

    // Create SuiKeyPair with proper format: [scheme_flag || private_key_bytes]
    let private_key = keypair.private();
    let private_key_bytes = private_key.as_bytes();
    let mut keypair_bytes = Vec::new();
    keypair_bytes.push(scheme_flag);
    keypair_bytes.extend_from_slice(private_key_bytes);

    let sui_keypair = SuiKeyPair::from_bytes(&keypair_bytes)
      .map_err(|e| Error::Internal(format!("Failed to create SuiKeyPair: {}", e)))?;

    keystore
      .add_key(None, sui_keypair)
      .map_err(|e| Error::Internal(format!("Failed to add key to keystore: {}", e)))?;

    // Verify the address matches
    if sui_address != sponsor_address {
      return Err(Error::Internal(format!(
        "Private key address {} doesn't match sponsor address {}",
        sui_address, sponsor_address
      )));
    }

    tracing::info!("Successfully imported sponsor private key for address: {}", sponsor_address);

    Ok(Self {
      app_state,
      gas_station: Some(Arc::new(gas_station)),
      sponsor_keystore: Some(Arc::new(keystore)),
    })
  }
}

#[async_trait]
impl SuiRepository for SuiRepositoryImpl {
  async fn fetch_coin(&self, sender: String) -> Result<Option<Coin>> {
    let coin_type = "0x2::sui::SUI".to_string();
    let address =
      SuiAddress::from_str(&sender).map_err(|e| Error::InvalidRequest(e.to_string()))?;
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

  async fn get_available_gas(&self, required_budget: u64) -> Result<ObjectID> {
    let gas_station = self
      .gas_station
      .as_ref()
      .ok_or_else(|| Error::Internal("Gas station not initialized".to_string()))?;

    gas_station
      .get_available_gas(required_budget)
      .await
      .map_err(|e| Error::Internal(e.to_string()))
  }

  async fn release_gas(&self, object_id: ObjectID) -> Result<()> {
    let gas_station = self
      .gas_station
      .as_ref()
      .ok_or_else(|| Error::Internal("Gas station not initialized".to_string()))?;

    gas_station.release_gas(object_id).await;
    Ok(())
  }

  // TODO: Not really working, need to finish later
  async fn sponsor_transaction(
    &self,
    tx_bytes: Vec<u8>,
    user_signature: &[u8],
  ) -> Result<(Transaction, String)> {
    // Check if we have gas station and keystore
    let gas_station = self
      .gas_station
      .as_ref()
      .ok_or_else(|| Error::Internal("Gas station not initialized".to_string()))?;

    let keystore = self.sponsor_keystore.as_ref().ok_or_else(|| {
      Error::Internal(
        "Sponsor keystore not available. Private key required for transaction sponsoring."
          .to_string(),
      )
    })?;

    // Parse transaction data from bytes
    let tx_data = match bcs::from_bytes::<TransactionData>(&tx_bytes) {
      Ok(data) => data,
      Err(e) => {
        tracing::error!("Failed to parse transaction data: {}", e);
        return Err(Error::Internal(format!("Failed to parse transaction data: {}", e)));
      }
    };

    // Extract sender and gas data using pattern matching
    let (_sender, gas_budget) = match &tx_data {
      TransactionData::V1(tx) => (tx.sender, tx.gas_data.budget),
    };

    let sponsor_address = gas_station.sponsor_address;

    // Get an available gas object for the required budget
    let _gas_object_id = match gas_station.get_available_gas(gas_budget).await {
      Ok(id) => id,
      Err(e) => {
        tracing::error!("Failed to get gas object: {}", e);
        return Err(Error::Internal(format!("Failed to get gas object: {}", e)));
      }
    };

    // Create a new transaction data with sponsor's gas object
    let sponsored_tx_data = tx_data.clone();

    // Create transaction bytes for signing
    let tx_bytes_for_signing = match bcs::to_bytes(&sponsored_tx_data) {
      Ok(bytes) => bytes,
      Err(e) => {
        tracing::error!("Failed to serialize transaction: {}", e);
        return Err(Error::Internal(format!("Failed to serialize transaction: {}", e)));
      }
    };

    // Sign the transaction with sponsor's key
    let keypair = match keystore.get_key(&sponsor_address) {
      Ok(kp) => kp,
      Err(e) => {
        tracing::error!("Failed to get keypair: {}", e);
        return Err(Error::Internal(format!("Failed to get keypair: {}", e)));
      }
    };

    let signature = keypair.sign(&tx_bytes_for_signing);

    // Parse the user signature from bytes
    // For sponsored transactions, we typically receive raw signature bytes from the frontend
    // We'll construct a proper signature or work with the bytes directly
    if user_signature.len() != 64 {
      tracing::error!(
        "Invalid user signature length: {} bytes (expected 64)",
        user_signature.len()
      );
      return Err(Error::Internal(
        "Invalid user signature length. Expected 64 bytes for Ed25519 signature.".to_string(),
      ));
    }

    // For now, we'll skip adding the user signature to the transaction
    // In a proper implementation, you would need to:
    // 1. Reconstruct the full signature with proper format (flag || sig || pk)
    // 2. Or handle multi-sig scenarios properly
    // 3. Ensure the transaction structure supports sponsored transactions

    // Add sponsor signature
    let all_signatures = vec![signature];

    // Create the final transaction
    let final_transaction = Transaction::from_data(sponsored_tx_data, all_signatures);

    // Submit the transaction to the Sui network
    let response = gas_station
      .sui_client
      .quorum_driver_api()
      .execute_transaction_block(
        final_transaction.clone(),
        sui_sdk::rpc_types::SuiTransactionBlockResponseOptions::full_content(),
        None,
      )
      .await
      .map_err(|e| Error::Internal(format!("Failed to submit transaction to network: {}", e)))?;

    tracing::info!("Transaction submitted successfully with digest: {}", response.digest);

    Ok((final_transaction, response.digest.to_string()))
  }

  async fn get_pool_stats(&self) -> Result<GasPoolStatus> {
    let gas_station = self
      .gas_station
      .as_ref()
      .ok_or_else(|| Error::Internal("Gas station not initialized".to_string()))?;

    Ok(gas_station.get_pool_stats().await)
  }

  async fn refresh_gas_pool(&self) -> Result<()> {
    let gas_station = self
      .gas_station
      .as_ref()
      .ok_or_else(|| Error::Internal("Gas station not initialized".to_string()))?;

    gas_station
      .refresh_gas_pool()
      .await
      .map_err(|e| Error::Internal(e.to_string()))
  }

  async fn log_sponsored_transaction(
    &self,
    user_address: &SuiAddress,
    gas_budget: u64,
  ) -> Result<()> {
    sqlx::query(
      "INSERT INTO sponsored_transactions (user_address, gas_budget, timestamp) VALUES ($1, $2, $3)"
    )
    .bind(user_address.to_string())
    .bind(gas_budget as i64)
    .bind(time::now_utc())
    .execute(self.app_state.mm().dbx().db())
    .await
    .map_err(|e| Error::Internal(e.to_string()))?;

    Ok(())
  }

  async fn get_user_stats(&self, address: &str) -> Result<Option<UserStats>> {
    let stats = sqlx::query_as::<_, UserStats>(
      "SELECT user_address, COUNT(*) as transaction_count, SUM(gas_budget) as total_gas_sponsored, MAX(timestamp) as last_transaction FROM sponsored_transactions WHERE user_address = $1 GROUP BY user_address"
    )
    .bind(address)
    .fetch_optional(self.app_state.mm().dbx().db())
    .await
    .map_err(|e| Error::Internal(e.to_string()))?;

    Ok(stats)
  }

  async fn check_rate_limit(&self, user_address: &SuiAddress) -> Result<bool> {
    let mut conn = self
      .app_state
      .redis()
      .get_multiplexed_async_connection()
      .await
      .map_err(|e| Error::Internal(e.to_string()))?;

    let key = format!("rate_limit:{}", user_address);

    let count: Option<i32> = conn
      .get(&key)
      .await
      .map_err(|e| Error::Internal(e.to_string()))?;

    match count {
      Some(c) if c >= 10 => Ok(false), // Rate limit exceeded
      Some(_) => {
        conn
          .incr::<_, _, ()>(&key, 1)
          .await
          .map_err(|e| Error::Internal(e.to_string()))?;
        Ok(true)
      }
      None => {
        conn
          .set_ex::<_, _, ()>(&key, 1, 60)
          .await
          .map_err(|e| Error::Internal(e.to_string()))?;
        Ok(true)
      }
    }
  }
}
