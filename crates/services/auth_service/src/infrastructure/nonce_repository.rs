use async_trait::async_trait;
use redis::AsyncCommands;
use crate::domain::Nonce;
use crate::error::{Error, Result};

#[async_trait]
pub trait NonceRepository: Send + Sync {
  async fn store_nonce(&self, nonce: &Nonce) -> Result<()>;
  async fn get_nonce(&self, address: &str) -> Result<Option<Nonce>>;
  async fn remove_nonce(&self, address: &str) -> Result<()>;
  async fn cleanup_expired_nonces(&self) -> Result<()>;
}

pub struct RedisNonceRepository {
  client: redis::Client,
}

impl RedisNonceRepository {
  pub fn new(redis_url: &str) -> Result<Self> {
    let client = redis::Client::open(redis_url)
      .map_err(|e| Error::redis_error(&format!("Failed to connect to Redis: {}", e)))?;
    
    Ok(Self { client })
  }

  fn nonce_key(address: &str) -> String {
    format!("auth:nonce:{}", address)
  }
}

#[async_trait]
impl NonceRepository for RedisNonceRepository {
  async fn store_nonce(&self, nonce: &Nonce) -> Result<()> {
    let mut conn = self.client.get_multiplexed_async_connection().await?;
    let key = Self::nonce_key(&nonce.address);
    let value = serde_json::to_string(nonce)
      .map_err(|e| Error::internal_error(&format!("Failed to serialize nonce: {}", e)))?;
    
    // Set with expiration (5 minutes)
    let _: () = conn.set_ex(&key, value, 300).await?;
    
    Ok(())
  }

  async fn get_nonce(&self, address: &str) -> Result<Option<Nonce>> {
    let mut conn = self.client.get_multiplexed_async_connection().await?;
    let key = Self::nonce_key(address);
    
    let value: Option<String> = conn.get(&key).await?;
    
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
    let mut conn = self.client.get_multiplexed_async_connection().await?;
    let key = Self::nonce_key(address);
    
    let _: () = conn.del(&key).await?;
    
    Ok(())
  }

  async fn cleanup_expired_nonces(&self) -> Result<()> {
    // Redis automatically handles expiration, so this is a no-op
    Ok(())
  }
} 