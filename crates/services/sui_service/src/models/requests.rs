use serde::{Deserialize, Serialize};
use sui_sdk::types::base_types::{ObjectID, SuiAddress, TransactionDigest};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetObjectRequest {
  pub object_id: ObjectID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCoinBalanceRequest {
  pub address: SuiAddress,
  pub coin_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDynamicFieldsRequest {
  pub parent_object_id: ObjectID,
  pub cursor: Option<ObjectID>,
  pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsRequest {
  pub cursor: Option<TransactionDigest>,
  pub limit: Option<usize>,
  pub descending_order: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsByTransactionRequest {
  pub digest: TransactionDigest,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsByModuleRequest {
  pub package: ObjectID,
  pub module: String,
  pub cursor: Option<TransactionDigest>,
  pub limit: Option<usize>,
}
