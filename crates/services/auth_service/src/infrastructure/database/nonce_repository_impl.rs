use async_trait::async_trait;
use jd_core::AppState;
use redis::AsyncCommands;
use serde_json;

use crate::domain::{Nonce, NonceRepository};
use crate::error::{Error, Result};

pub struct NonceRepositoryImpl {
  state: AppState,
}

impl NonceRepositoryImpl {
  pub fn new(state: AppState) -> Self {
    Self { state }
  }

  fn nonce_key(address: &str) -> String {
    format!("auth:nonce:{}", address)
  }
}

#[async_trait]
impl NonceRepository for NonceRepositoryImpl {
  async fn store_nonce(&self, nonce: &Nonce) -> Result<()> {
    let mut conn = self
      .state
      .redis
      .get_multiplexed_async_connection()
      .await
      .map_err(|e| Error::internal_error(&format!("Failed to get Redis connection: {}", e)))?;

    let key = Self::nonce_key(&nonce.address);
    let value = serde_json::to_string(nonce)
      .map_err(|e| Error::internal_error(&format!("Failed to serialize nonce: {}", e)))?;

    // Set with expiration (5 minutes)
    let _: () = conn
      .set_ex(&key, value, 300)
      .await
      .map_err(|e| Error::internal_error(&format!("Failed to store nonce: {}", e)))?;

    Ok(())
  }

  async fn get_nonce(&self, address: &str) -> Result<Option<Nonce>> {
    let mut conn = self
      .state
      .redis
      .get_multiplexed_async_connection()
      .await
      .map_err(|e| Error::internal_error(&format!("Failed to get Redis connection: {}", e)))?;

    let key = Self::nonce_key(address);

    let value: Option<String> = conn
      .get(&key)
      .await
      .map_err(|e| Error::internal_error(&format!("Failed to get nonce: {}", e)))?;

    match value {
      Some(json) => {
        let nonce: Nonce = serde_json::from_str(&json)
          .map_err(|e| Error::internal_error(&format!("Failed to deserialize nonce: {}", e)))?;
        Ok(Some(nonce))
      }
      None => Ok(None),
    }
  }

  async fn remove_nonce(&self, address: &str) -> Result<()> {
    let mut conn = self
      .state
      .redis
      .get_multiplexed_async_connection()
      .await
      .map_err(|e| Error::internal_error(&format!("Failed to get Redis connection: {}", e)))?;

    let key = Self::nonce_key(address);

    let _: () = conn
      .del(&key)
      .await
      .map_err(|e| Error::internal_error(&format!("Failed to remove nonce: {}", e)))?;

    Ok(())
  }
}
