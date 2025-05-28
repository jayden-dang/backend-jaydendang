use std::sync::Arc;

use jd_storage::{dbx::Dbx, new_db_pool};
use redis::Client as RedisClient;

pub mod ctx;
mod error;
pub use error::{Error, Result};
pub mod base;

#[derive(Clone)]
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
  // TODO: S3 Service
  // TODO: Email Service
}

impl AppState {
  pub async fn new() -> Result<Self> {
    let mm = Arc::new(ModelManager::new().await?);
    let redis = Arc::new(RedisClient::open("redis://127.0.0.1/")?);

    Ok(AppState { mm, redis })
  }

  // Convenience methods
  pub fn mm(&self) -> &ModelManager {
    &self.mm
  }

  pub fn redis(&self) -> &RedisClient {
    &self.redis
  }
}
