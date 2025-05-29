use crate::infrastructure::sui_repository_impl::SuiRepositoryImpl;
use crate::models::{GasPoolStatus, SponsorRequest, SponsorResponse, UserStats};
use crate::Result;
use axum::{
  extract::{Path, State},
  Json,
};
use jd_core::AppState;
use jd_utils::config::Config;
use serde_json::{json, Value};
use std::str::FromStr;
use sui_sdk::rpc_types::Coin;
use sui_types::base_types::SuiAddress;

use crate::{
  application::use_cases::sui_use_cases::SuiUseCases, domain::sui_repository_trait::SuiRepository,
};

pub struct SuiHandler<R: SuiRepository> {
  pub use_cases: SuiUseCases<R>,
}

impl<R: SuiRepository> SuiHandler<R> {
  pub fn new(use_cases: SuiUseCases<R>) -> Self {
    Self { use_cases }
  }

  // Helper function to create repository with gas station if available
  async fn create_repository_with_gas_station(state: AppState) -> Result<SuiRepositoryImpl> {
    let config = Config::from_env()
      .map_err(|e| crate::error::Error::Internal(format!("Config error: {}", e)))?;

    if let Some(sponsor_addr_str) = &config.sui.sponsor_address {
      let sponsor_address = SuiAddress::from_str(sponsor_addr_str).map_err(|e| {
        crate::error::Error::InvalidRequest(format!(
          "Invalid sponsor address '{}': {}",
          sponsor_addr_str, e
        ))
      })?;

      let max_gas_budget = config.sui.max_gas_budget.unwrap_or(1_000_000_000); // Default 1 SUI

      // Get RPC URL based on environment
      let rpc_url = match config.sui.env.to_lowercase().as_str() {
        "mainnet" => "https://fullnode.mainnet.sui.io:443",
        "testnet" => "https://fullnode.testnet.sui.io:443",
        "devnet" => "https://fullnode.devnet.sui.io:443",
        "local" => "http://127.0.0.1:9000",
        _ => {
          return Err(crate::error::Error::Internal(format!(
            "Unsupported Sui environment: {}",
            config.sui.env
          )))
        }
      };

      // Check if we have private key for full sponsor functionality
      if let Some(private_key) = &config.sui.sponsor_private_key {
        SuiRepositoryImpl::with_gas_station_and_key(
          state,
          rpc_url,
          sponsor_address,
          private_key,
          max_gas_budget,
        )
        .await
      } else {
        SuiRepositoryImpl::with_gas_station(state, rpc_url, sponsor_address, max_gas_budget).await
      }
    } else {
      Err(crate::error::Error::Internal(
        "Gas station requires SUI_SPONSOR_ADDRESS environment variable".to_string(),
      ))
    }
  }

  pub async fn fetch_coin(
    State(state): State<AppState>,
    Json(req): Json<String>,
  ) -> Result<Json<Coin>> {
    let repository = SuiRepositoryImpl::new(state);
    let use_cases = SuiUseCases::new(repository);
    let object = use_cases.fetch_coin(req).await?;
    Ok(Json(object))
  }

  // Gas Station Handlers
  pub async fn health_check() -> &'static str {
    "Gas Station is healthy"
  }

  // Debug endpoint to check configuration
  pub async fn debug_config() -> Result<Json<Value>> {
    let config = Config::from_env()
      .map_err(|e| crate::error::Error::Internal(format!("Config error: {}", e)))?;

    let debug_info = json!({
      "sui_env": config.sui.env,
      "sponsor_address_set": config.sui.sponsor_address.is_some(),
      "sponsor_address": config.sui.sponsor_address.as_deref().unwrap_or("NOT_SET"),
      "sponsor_private_key_set": config.sui.sponsor_private_key.is_some(),
      "max_gas_budget": config.sui.max_gas_budget.unwrap_or(1_000_000_000),
      "required_env_vars": {
        "SUI.ENV": std::env::var("SUI.ENV").unwrap_or("NOT_SET".to_string()),
        "SUI.SPONSOR_ADDRESS": if std::env::var("SUI.SPONSOR_ADDRESS").is_ok() { "SET" } else { "NOT_SET" },
        "SUI.SPONSOR_PRIVATE_KEY": if std::env::var("SUI.SPONSOR_PRIVATE_KEY").is_ok() { "SET" } else { "NOT_SET" },
        "SUI.MAX_GAS_BUDGET": std::env::var("SUI.MAX_GAS_BUDGET").unwrap_or("NOT_SET".to_string()),
      },
      "sponsor_capabilities": {
        "gas_pool_management": config.sui.sponsor_address.is_some(),
        "transaction_sponsoring": config.sui.sponsor_address.is_some() && config.sui.sponsor_private_key.is_some()
      }
    });

    Ok(Json(debug_info))
  }

  pub async fn sponsor_transaction(
    State(state): State<AppState>,
    Json(request): Json<SponsorRequest>,
  ) -> Result<Json<SponsorResponse>> {
    let repository = Self::create_repository_with_gas_station(state).await?;
    let use_cases = SuiUseCases::new(repository);
    let response = use_cases.sponsor_transaction(request).await?;
    Ok(Json(response))
  }

  pub async fn gas_pool_status(State(state): State<AppState>) -> Result<Json<GasPoolStatus>> {
    let repository = Self::create_repository_with_gas_station(state).await?;
    let use_cases = SuiUseCases::new(repository);
    let status = use_cases.get_gas_pool_status().await?;
    Ok(Json(status))
  }

  pub async fn user_stats(
    State(state): State<AppState>,
    Path(address): Path<String>,
  ) -> Result<Json<UserStats>> {
    let repository = Self::create_repository_with_gas_station(state).await?;
    let use_cases = SuiUseCases::new(repository);
    let stats = use_cases.get_user_stats(address).await?;
    Ok(Json(stats))
  }

  pub async fn refresh_gas_pool(State(state): State<AppState>) -> Result<Json<String>> {
    let repository = Self::create_repository_with_gas_station(state).await?;
    let use_cases = SuiUseCases::new(repository);
    use_cases.refresh_gas_pool().await?;
    Ok(Json("Gas pool refreshed successfully".to_string()))
  }

  // Test endpoint to validate gas station setup
  pub async fn gas_station(State(state): State<AppState>) -> Result<Json<Value>> {
    tracing::info!("Testing gas station initialization...");

    let config = Config::from_env()
      .map_err(|e| crate::error::Error::Internal(format!("Config error: {}", e)))?;

    if let Some(sponsor_addr_str) = &config.sui.sponsor_address {
      match SuiAddress::from_str(sponsor_addr_str) {
        Ok(sponsor_address) => {
          let max_gas_budget = config.sui.max_gas_budget.unwrap_or(1_000_000_000);

          let rpc_url = match config.sui.env.to_lowercase().as_str() {
            "mainnet" => "https://fullnode.mainnet.sui.io:443",
            "testnet" => "https://fullnode.testnet.sui.io:443",
            "devnet" => "https://fullnode.devnet.sui.io:443",
            "local" => "http://127.0.0.1:9000",
            _ => {
              return Ok(Json(json!({
                "status": "error",
                "message": format!("Unsupported Sui environment: {}", config.sui.env)
              })))
            }
          };

          tracing::info!(
            "Attempting to create gas station with sponsor: {}, rpc: {}",
            sponsor_address,
            rpc_url
          );

          match SuiRepositoryImpl::with_gas_station(state, rpc_url, sponsor_address, max_gas_budget)
            .await
          {
            Ok(repository) => {
              // Try to get gas pool status
              match repository.get_pool_stats().await {
                Ok(stats) => Ok(Json(json!({
                  "status": "success",
                  "sponsor_address": sponsor_addr_str,
                  "rpc_url": rpc_url,
                  "gas_pool_stats": {
                    "total_objects": stats.total_objects,
                    "total_balance": stats.total_balance,
                    "available_objects": stats.available_objects,
                    "utilization_rate": stats.utilization_rate
                  }
                }))),
                Err(e) => Ok(Json(json!({
                  "status": "error",
                  "message": format!("Failed to get gas pool stats: {}", e),
                  "sponsor_address": sponsor_addr_str,
                  "rpc_url": rpc_url
                }))),
              }
            }
            Err(e) => Ok(Json(json!({
              "status": "error",
              "message": format!("Failed to create gas station: {}", e),
              "sponsor_address": sponsor_addr_str,
              "rpc_url": rpc_url
            }))),
          }
        }
        Err(e) => Ok(Json(json!({
          "status": "error",
          "message": format!("Invalid sponsor address '{}': {}", sponsor_addr_str, e)
        }))),
      }
    } else {
      Ok(Json(json!({
        "status": "error",
        "message": "SUI_SPONSOR_ADDRESS not configured"
      })))
    }
  }
}
