use std::sync::Arc;
use tracing::info;
pub mod sui;

use jd_storage::{dbx::Dbx, new_db_pool};
use jd_utils::config::Config;
use redis::Client as RedisClient;

pub mod ctx;
mod error;
pub use error::{Error, Result};
pub mod base;
pub use base::error as service_error;

#[derive(Clone, rpc_router::RpcResource)]
pub struct ModelManager {
  dbx: Dbx,
}

impl ModelManager {
  pub async fn new() -> Result<Self> {
    let db_pool = new_db_pool()
      .await
      .map_err(|ex| Error::CantCreateModelManagerProvider(ex.to_string()))?;
    let dbx = Dbx::new(db_pool, true)?;
    Ok(ModelManager { dbx })
  }

  pub fn new_with_txn(&self) -> Result<ModelManager> {
    let dbx = Dbx::new(self.dbx.db().clone(), true)?;
    Ok(ModelManager { dbx })
  }

  pub fn dbx(&self) -> &Dbx {
    &self.dbx
  }
}

#[derive(Clone)]
pub struct AppState {
  pub mm: Arc<ModelManager>,
  pub redis: Arc<RedisClient>,
  pub sui_client: Arc<sui::sui_client::SuiClient>,
  pub config: Arc<Config>,
  // TODO: S3 Service
  // TODO: Email Service
}

impl AppState {
  pub async fn new() -> Result<Self> {
    let mm = Arc::new(ModelManager::new().await?);
    let config = Arc::new(Config::from_env()?);

    let redis = Arc::new(RedisClient::open(config.redis.addr.clone())?);

    info!("Initializing Sui client with environment: {}", config.sui.env);
    let sui_client = Arc::new(
      sui::sui_client::SuiClient::new(&config.sui)
        .await
        .map_err(|ex| Error::CantCreateSuiClient(ex.to_string()))?,
    );

    Ok(AppState { mm, redis, sui_client, config })
  }

  // Convenience methods
  pub fn mm(&self) -> &ModelManager {
    &self.mm
  }

  pub fn redis(&self) -> &RedisClient {
    &self.redis
  }

  pub fn sui_client(&self) -> &sui::sui_client::SuiClient {
    &self.sui_client
  }
}
