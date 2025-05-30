use crate::Result;
use crate::error::Error;
use crate::models::{GasPoolStatus, SponsorRequest, SponsorResponse, UserStats};
use std::str::FromStr;
use sui_sdk::rpc_types::Coin;
use sui_types::base_types::{ObjectID, SuiAddress};
use uuid::Uuid;

use crate::domain::sui_repository_trait::SuiRepository;

#[derive(Clone)]
pub struct SuiUseCases<R: SuiRepository> {
  pub repository: R,
}

impl<R: SuiRepository> SuiUseCases<R> {
  pub fn new(repository: R) -> Self {
    Self { repository }
  }

  pub async fn fetch_coin(&self, sender: String) -> Result<Coin> {
    self
      .repository
      .fetch_coin(sender)
      .await?
      .ok_or_else(|| Error::InvalidRequest("Coin not found".into()))
  }

  // Gas Station Use Cases
  pub async fn sponsor_transaction(&self, request: SponsorRequest) -> Result<SponsorResponse> {
    // Parse user address
    let user_address = SuiAddress::from_str(&request.user_address)
      .map_err(|_| Error::InvalidRequest("Invalid user address".to_string()))?;

    // Check rate limiting
    if !self.repository.check_rate_limit(&user_address).await? {
      return Err(Error::InvalidRequest("Rate limit exceeded".to_string()));
    }

    // Sponsor the transaction using tx_bytes
    match self
      .repository
      .sponsor_transaction(request.transaction_data.tx_bytes, &request.user_signature)
      .await
    {
      Ok((transaction, digest)) => {
        // Get gas budget from request for logging
        let gas_budget = request
          .transaction_data
          .gas_data
          .budget
          .parse::<u64>()
          .unwrap_or(10000000); // Default gas budget

        // Log the sponsored transaction
        if let Err(e) = self
          .repository
          .log_sponsored_transaction(&user_address, gas_budget)
          .await
        {
          tracing::warn!("Failed to log sponsored transaction: {}", e);
        }

        Ok(SponsorResponse {
          sponsored_transaction: Some(transaction),
          sponsored_tx_bytes: None,
          sponsored_tx_digest: Some(digest.clone()),
          transaction_id: Uuid::new_v4(),
          status: "success".to_string(),
          message: Some(format!(
            "Transaction executed successfully on Sui network. Digest: {}",
            digest
          )),
        })
      }
      Err(Error::ImplementationPending(msg)) => {
        // Return success response for development phase without creating a mock transaction
        Ok(SponsorResponse {
          sponsored_transaction: None,
          sponsored_tx_bytes: None,
          sponsored_tx_digest: None,
          transaction_id: Uuid::new_v4(),
          status: "development_success".to_string(),
          message: Some(format!("Development Phase: {}", msg)),
        })
      }
      Err(e) => {
        // Provide helpful error messages for common issues
        let error_message = match e {
                Error::Internal(ref msg) if msg.contains("User signature is required") => {
                    "User signature is required for sponsored transactions. Please sign the transaction with your wallet first, then submit both the transaction data and your signature.".to_string()
                },
                Error::Internal(ref msg) if msg.contains("Invalid user signature") => {
                    "Invalid user signature format. Please ensure the signature is properly formatted.".to_string()
                },
                _ => format!("Failed to sponsor transaction: {}", e)
            };

        Err(Error::Internal(error_message))
      }
    }
  }

  pub async fn get_gas_pool_status(&self) -> Result<GasPoolStatus> {
    self.repository.get_pool_stats().await
  }

  pub async fn get_user_stats(&self, address: String) -> Result<UserStats> {
    let stats = self.repository.get_user_stats(&address).await?;

    match stats {
      Some(stats) => Ok(stats),
      None => Ok(UserStats::default_for_address(address)),
    }
  }

  pub async fn refresh_gas_pool(&self) -> Result<()> {
    self.repository.refresh_gas_pool().await
  }

  pub async fn get_available_gas(&self, required_budget: u64) -> Result<ObjectID> {
    self.repository.get_available_gas(required_budget).await
  }

  pub async fn release_gas(&self, object_id: ObjectID) -> Result<()> {
    self.repository.release_gas(object_id).await
  }
}
