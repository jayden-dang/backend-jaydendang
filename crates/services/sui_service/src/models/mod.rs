use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use chrono::{DateTime, Utc};
use sui_sdk::types::{base_types::ObjectID, dynamic_field::DynamicFieldInfo, object::Data};
use sui_types::transaction::Transaction;
use uuid::Uuid;

pub mod requests;

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinBalance {
  pub coin_type: String,
  pub coin_object_count: u64,
  pub total_balance: u64,
  pub locked_balance: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectInfo {
  pub object_id: ObjectID,
  pub version: u64,
  pub digest: String,
  pub type_: String,
  pub owner: String,
  pub previous_transaction: String,
  pub storage_rebate: u64,
  pub content: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicFieldPage {
  pub data: Vec<DynamicFieldInfo>,
  pub next_cursor: Option<ObjectID>,
  pub has_next_page: bool,
}

// Gas Station Models - Custom JSON-friendly structs
#[derive(Debug, Serialize, Deserialize)]
pub struct GasData {
  pub budget: String, // JSON usually sends numbers as strings
  pub price: String,  // JSON usually sends numbers as strings
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionDataJson {
  pub sender: String,
  pub tx_bytes: Vec<u8>,
  pub gas_data: GasData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SponsorRequest {
  pub user_address: String,
  pub transaction_data: TransactionDataJson,
  pub user_signature: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SponsorResponse {
  pub sponsored_transaction: Option<Transaction>,
  pub sponsored_tx_bytes: Option<Vec<u8>>,
  pub sponsored_tx_digest: Option<String>,
  pub transaction_id: Uuid,
  pub status: String,
  pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasPoolStatus {
  pub total_objects: usize,
  pub total_balance: u64,
  pub available_objects: usize,
  pub utilization_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserStats {
  pub user_address: String,
  pub transaction_count: Option<i64>,
  pub total_gas_sponsored: Option<i64>,
  pub last_transaction: Option<DateTime<Utc>>,
}

impl UserStats {
  pub fn default_for_address(address: String) -> Self {
    Self {
      user_address: address,
      transaction_count: Some(0),
      total_gas_sponsored: Some(0),
      last_transaction: None,
    }
  }
}
