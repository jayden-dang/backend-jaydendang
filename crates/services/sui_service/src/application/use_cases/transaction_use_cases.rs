use crate::domain::sui_repository_trait::SuiRepository;
use crate::{Result, error::Error};
use sui_sdk::rpc_types::{SuiTransactionBlockResponse, SuiTransactionBlockResponseOptions};
use sui_sdk::types::base_types::TransactionDigest;
use std::str::FromStr;

/// Use cases for transaction operations on Sui blockchain
#[derive(Clone)]
pub struct TransactionUseCases<R: SuiRepository> {
  repository: R,
}

impl<R: SuiRepository> TransactionUseCases<R> {
  pub fn new(repository: R) -> Self {
    Self { repository }
  }

  /// Get detailed transaction information
  pub async fn get_transaction_details(&self, digest: &str) -> Result<SuiTransactionBlockResponse> {
    let tx_digest = TransactionDigest::from_str(digest)
      .map_err(|_| Error::InvalidRequest("Invalid transaction digest format".to_string()))?;

    let options = Some(SuiTransactionBlockResponseOptions {
      show_input: true,
      show_raw_input: false,
      show_effects: true,
      show_events: true,
      show_object_changes: true,
      show_balance_changes: true,
      show_raw_effects: false,
    });

    self.repository.get_transaction_block(tx_digest, options).await
  }

  /// Get basic transaction information (minimal data)
  pub async fn get_transaction_basic(&self, digest: &str) -> Result<SuiTransactionBlockResponse> {
    let tx_digest = TransactionDigest::from_str(digest)
      .map_err(|_| Error::InvalidRequest("Invalid transaction digest format".to_string()))?;

    let options = Some(SuiTransactionBlockResponseOptions {
      show_input: false,
      show_raw_input: false,
      show_effects: true,
      show_events: false,
      show_object_changes: false,
      show_balance_changes: false,
      show_raw_effects: false,
    });

    self.repository.get_transaction_block(tx_digest, options).await
  }

  /// Get multiple transactions at once
  pub async fn get_transactions_batch(
    &self,
    digests: Vec<String>,
  ) -> Result<Vec<SuiTransactionBlockResponse>> {
    let tx_digests: Result<Vec<TransactionDigest>> = digests
      .into_iter()
      .map(|digest| {
        TransactionDigest::from_str(&digest)
          .map_err(|_| Error::InvalidRequest(format!("Invalid transaction digest: {}", digest)))
      })
      .collect();

    let tx_digests = tx_digests?;

    let options = Some(SuiTransactionBlockResponseOptions {
      show_input: true,
      show_raw_input: false,
      show_effects: true,
      show_events: true,
      show_object_changes: true,
      show_balance_changes: true,
      show_raw_effects: false,
    });

    self.repository.get_transaction_blocks(tx_digests, options).await
  }
}