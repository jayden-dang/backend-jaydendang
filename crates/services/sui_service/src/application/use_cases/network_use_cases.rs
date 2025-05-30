use crate::domain::sui_repository_trait::SuiRepository;
use crate::Result;

/// Use cases for network operations on Sui blockchain
#[derive(Clone)]
pub struct NetworkUseCases<R: SuiRepository> {
  repository: R,
}

impl<R: SuiRepository> NetworkUseCases<R> {
  pub fn new(repository: R) -> Self {
    Self { repository }
  }

  /// Get the latest checkpoint sequence number
  pub async fn get_latest_checkpoint(&self) -> Result<u64> {
    self.repository.get_latest_checkpoint_sequence_number().await
  }

  /// Get total number of transactions on the network
  pub async fn get_total_transactions(&self) -> Result<u64> {
    self.repository.get_total_transaction_blocks().await
  }

  /// Get current reference gas price
  pub async fn get_gas_price(&self) -> Result<u64> {
    self.repository.get_reference_gas_price().await
  }

  /// Get chain identifier
  pub async fn get_chain_id(&self) -> Result<String> {
    self.repository.get_chain_identifier().await
  }
}