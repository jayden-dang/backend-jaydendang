use crate::error::Error;
use crate::Result;
use jd_utils::config::SuiConfig;
use sui_sdk::SuiClientBuilder;

pub struct SuiClient {
  pub client: sui_sdk::SuiClient,
}

impl SuiClient {
  pub async fn new(config: &SuiConfig) -> Result<Self> {
    let client = match config.env.to_lowercase().as_str() {
      "mainnet" => SuiClientBuilder::default().build_mainnet().await?,
      "testnet" => SuiClientBuilder::default().build_testnet().await?,
      "devnet" => SuiClientBuilder::default().build_devnet().await?,
      "local" => SuiClientBuilder::default().build_localnet().await?,
      _ => {
        return Err(Error::CantCreateSuiClient(format!("Invalid Sui environment: {}", config.env)));
      }
    };

    Ok(Self { client })
  }

  pub async fn get_api_version(&self) -> Result<String> {
    Ok(self.client.api_version().to_string())
  }
}
