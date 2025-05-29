use anyhow::Result;
use sui_sdk::SuiClientBuilder;

pub struct SuiClient {
    client: sui_sdk::SuiClient,
}

impl SuiClient {
    pub async fn new() -> Result<Self> {
        // Connect to Sui testnet by default
        let client = SuiClientBuilder::default()
            .build_testnet()
            .await?;
        
        Ok(Self { client })
    }

    pub async fn get_api_version(&self) -> Result<String> {
        Ok(self.client.api_version())
    }

    // Add more methods for blockchain interactions here
    // For example:
    // - get_balance
    // - get_transaction
    // - get_object
    // etc.
} 