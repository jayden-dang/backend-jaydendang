use crate::domain::sui_repository_trait::SuiRepository;
use crate::{Result, error::Error};
use sui_sdk::rpc_types::{Coin, Balance, SuiCoinMetadata, Page};
use sui_sdk::types::base_types::SuiAddress;
use std::str::FromStr;

/// Use cases for coin operations on Sui blockchain
#[derive(Clone)]
pub struct CoinUseCases<R: SuiRepository> {
  repository: R,
}

impl<R: SuiRepository> CoinUseCases<R> {
  pub fn new(repository: R) -> Self {
    Self { repository }
  }

  /// Get SUI coins for an address
  pub async fn get_sui_coins(
    &self,
    address: &str,
    limit: Option<usize>,
  ) -> Result<Page<Coin, String>> {
    let sui_address = SuiAddress::from_str(address)
      .map_err(|_| Error::InvalidRequest("Invalid address format".to_string()))?;
    
    self.repository
      .get_coins(sui_address, Some("0x2::sui::SUI".to_string()), None, limit)
      .await
  }

  /// Get all coins for an address
  pub async fn get_all_coins(
    &self,
    address: &str,
    limit: Option<usize>,
  ) -> Result<Page<Coin, String>> {
    let sui_address = SuiAddress::from_str(address)
      .map_err(|_| Error::InvalidRequest("Invalid address format".to_string()))?;
    
    self.repository.get_all_coins(sui_address, None, limit).await
  }

  /// Get coins for a specific coin type
  pub async fn get_coins_by_type(
    &self,
    address: &str,
    coin_type: &str,
    limit: Option<usize>,
  ) -> Result<Page<Coin, String>> {
    let sui_address = SuiAddress::from_str(address)
      .map_err(|_| Error::InvalidRequest("Invalid address format".to_string()))?;
    
    self.repository
      .get_coins(sui_address, Some(coin_type.to_string()), None, limit)
      .await
  }

  /// Get SUI balance for an address
  pub async fn get_sui_balance(&self, address: &str) -> Result<Balance> {
    let sui_address = SuiAddress::from_str(address)
      .map_err(|_| Error::InvalidRequest("Invalid address format".to_string()))?;
    
    self.repository.get_balance(sui_address, None).await
  }

  /// Get balance for a specific coin type
  pub async fn get_balance_by_type(&self, address: &str, coin_type: &str) -> Result<Balance> {
    let sui_address = SuiAddress::from_str(address)
      .map_err(|_| Error::InvalidRequest("Invalid address format".to_string()))?;
    
    self.repository
      .get_balance(sui_address, Some(coin_type.to_string()))
      .await
  }

  /// Get all balances for an address
  pub async fn get_all_balances(&self, address: &str) -> Result<Vec<Balance>> {
    let sui_address = SuiAddress::from_str(address)
      .map_err(|_| Error::InvalidRequest("Invalid address format".to_string()))?;
    
    self.repository.get_all_balances(sui_address).await
  }

  /// Get coin metadata
  pub async fn get_coin_metadata(&self, coin_type: &str) -> Result<Option<SuiCoinMetadata>> {
    self.repository.get_coin_metadata(coin_type.to_string()).await
  }

  /// Get total supply for a coin
  pub async fn get_total_supply(&self, coin_type: &str) -> Result<Option<u64>> {
    self.repository.get_total_supply(coin_type.to_string()).await
  }

  /// Select coins for a transaction amount
  pub async fn select_coins_for_amount(
    &self,
    address: &str,
    coin_type: Option<&str>,
    amount: u64,
  ) -> Result<Vec<Coin>> {
    let sui_address = SuiAddress::from_str(address)
      .map_err(|_| Error::InvalidRequest("Invalid address format".to_string()))?;
    
    let coin_type_str = coin_type.map(|s| s.to_string());
    
    self.repository
      .select_coins(sui_address, coin_type_str, amount, vec![])
      .await
  }

  /// Check if an address has sufficient balance
  pub async fn has_sufficient_balance(
    &self,
    address: &str,
    coin_type: Option<&str>,
    required_amount: u64,
  ) -> Result<bool> {
    let balance = match coin_type {
      Some(coin_type) => self.get_balance_by_type(address, coin_type).await?,
      None => self.get_sui_balance(address).await?,
    };

    Ok(balance.total_balance >= required_amount as u128)
  }

  /// Get formatted balance information
  pub async fn get_balance_info(&self, address: &str) -> Result<BalanceInfo> {
    let all_balances = self.get_all_balances(address).await?;
    
    let mut sui_balance = 0u128;
    let mut other_coins = Vec::new();
    
    for balance in all_balances {
      if balance.coin_type == "0x2::sui::SUI" {
        sui_balance = balance.total_balance;
      } else {
        other_coins.push(CoinBalanceInfo {
          coin_type: balance.coin_type,
          balance: balance.total_balance,
          coin_object_count: balance.coin_object_count,
        });
      }
    }
    
    Ok(BalanceInfo {
      address: address.to_string(),
      sui_balance,
      other_coins,
    })
  }
}

/// Balance information for an address
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BalanceInfo {
  pub address: String,
  pub sui_balance: u128,
  pub other_coins: Vec<CoinBalanceInfo>,
}

/// Information about a specific coin balance
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoinBalanceInfo {
  pub coin_type: String,
  pub balance: u128,
  pub coin_object_count: usize,
}