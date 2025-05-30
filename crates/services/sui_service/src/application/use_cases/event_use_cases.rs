use crate::domain::sui_repository_trait::SuiRepository;
use crate::{Result, error::Error};
use sui_sdk::rpc_types::SuiEvent;
use sui_sdk::types::base_types::TransactionDigest;
use std::str::FromStr;

/// Use cases for event operations on Sui blockchain
#[derive(Clone)]
pub struct EventUseCases<R: SuiRepository> {
  repository: R,
}

impl<R: SuiRepository> EventUseCases<R> {
  pub fn new(repository: R) -> Self {
    Self { repository }
  }

  /// Get events for a specific transaction
  pub async fn get_transaction_events(&self, digest: &str) -> Result<Vec<SuiEvent>> {
    let tx_digest = TransactionDigest::from_str(digest)
      .map_err(|_| Error::InvalidRequest("Invalid transaction digest format".to_string()))?;

    self.repository.get_events(tx_digest).await
  }
}