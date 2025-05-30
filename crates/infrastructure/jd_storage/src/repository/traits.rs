use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Base repository trait for CRUD operations
#[async_trait]
pub trait Repository<T, ID>: Send + Sync
where
  T: Send + Sync,
  ID: Send + Sync + Clone + 'static,
{
  type Error: std::error::Error + Send + Sync;

  /// Find an entity by its ID
  async fn find_by_id(&self, id: ID) -> Result<Option<T>, Self::Error>;

  /// Find all entities
  async fn find_all(&self) -> Result<Vec<T>, Self::Error>;

  /// Save a new entity
  async fn save(&self, entity: &T) -> Result<T, Self::Error>;

  /// Update an existing entity
  async fn update(&self, id: ID, entity: &T) -> Result<T, Self::Error>;

  /// Delete an entity by its ID
  async fn delete(&self, id: ID) -> Result<bool, Self::Error>;

  /// Check if an entity exists by its ID
  async fn exists(&self, id: ID) -> Result<bool, Self::Error>
  where
    ID: 'async_trait,
  {
    Ok(self.find_by_id(id).await?.is_some())
  }

  /// Count all entities
  async fn count(&self) -> Result<i64, Self::Error>;
}

/// Repository trait with filtering capabilities
#[async_trait]
pub trait FilterableRepository<T, ID, F>: Repository<T, ID>
where
  T: Send + Sync,
  ID: Send + Sync + Clone + 'static,
  F: Send + Sync,
{
  /// Find entities matching the given filter
  async fn find_by_filter(&self, filter: F) -> Result<Vec<T>, Self::Error>;

  /// Count entities matching the given filter
  async fn count_by_filter(&self, filter: F) -> Result<i64, Self::Error>;

  /// Delete entities matching the given filter
  async fn delete_by_filter(&self, filter: F) -> Result<u64, Self::Error>;
}

/// Repository trait with pagination support
#[async_trait]
pub trait PaginatedRepository<T, ID>: Repository<T, ID>
where
  T: Send + Sync,
  ID: Send + Sync + Clone + 'static,
{
  /// Find entities with pagination
  async fn find_paginated(
    &self,
    page: u64,
    per_page: u64,
  ) -> Result<PaginatedResult<T>, Self::Error>;
}

/// Repository trait with both filtering and pagination
#[async_trait]
pub trait FilterablePaginatedRepository<T, ID, F>:
  FilterableRepository<T, ID, F> + PaginatedRepository<T, ID>
where
  T: Send + Sync,
  ID: Send + Sync + Clone + 'static,
  F: Send + Sync,
{
  /// Find entities matching the filter with pagination
  async fn find_by_filter_paginated(
    &self,
    filter: F,
    page: u64,
    per_page: u64,
  ) -> Result<PaginatedResult<T>, Self::Error>;
}

/// Repository trait for entities with owner
#[async_trait]
pub trait OwnedRepository<T, ID, OwnerID>: Repository<T, ID>
where
  T: Send + Sync,
  ID: Send + Sync + Clone + 'static,
  OwnerID: Send + Sync,
{
  /// Find entities by owner ID
  async fn find_by_owner(&self, owner_id: OwnerID) -> Result<Vec<T>, Self::Error>;

  /// Find a specific entity by ID and owner ID
  async fn find_by_id_and_owner(&self, id: ID, owner_id: OwnerID)
  -> Result<Option<T>, Self::Error>;

  /// Delete an entity by ID and owner ID
  async fn delete_by_id_and_owner(&self, id: ID, owner_id: OwnerID) -> Result<bool, Self::Error>;
}

/// Result for paginated queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
  pub items: Vec<T>,
  pub total: i64,
  pub page: u64,
  pub per_page: u64,
  pub total_pages: u64,
}

impl<T> PaginatedResult<T> {
  pub fn new(items: Vec<T>, total: i64, page: u64, per_page: u64) -> Self {
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u64;
    Self { items, total, page, per_page, total_pages }
  }
}

/// Repository trait for caching support
#[async_trait]
pub trait CachedRepository<T, ID>: Repository<T, ID>
where
  T: Send + Sync + Serialize + for<'de> Deserialize<'de>,
  ID: Send + Sync + ToString + Clone + 'static,
{
  /// Get cache key for an entity
  fn cache_key(&self, id: &ID) -> String {
    format!("{}:{}", std::any::type_name::<T>(), id.to_string())
  }

  /// Get entity from cache
  async fn get_from_cache(&self, id: &ID) -> Result<Option<T>, Self::Error>;

  /// Set entity in cache
  async fn set_in_cache(&self, id: &ID, entity: &T, ttl: Option<u64>) -> Result<(), Self::Error>;

  /// Remove entity from cache
  async fn remove_from_cache(&self, id: &ID) -> Result<(), Self::Error>;

  /// Find by ID with cache support
  async fn find_by_id_cached(&self, id: ID) -> Result<Option<T>, Self::Error>
  where
    ID: 'async_trait,
  {
    // Try cache first
    if let Some(cached) = self.get_from_cache(&id).await? {
      return Ok(Some(cached));
    }

    // Fallback to database
    let id_clone = id.clone();
    if let Some(entity) = self.find_by_id(id_clone).await? {
      // Cache the result
      let _ = self.set_in_cache(&id, &entity, None).await;
      Ok(Some(entity))
    } else {
      Ok(None)
    }
  }
}

/// Repository trait for audit support
#[async_trait]
pub trait AuditableRepository<T, ID>: Repository<T, ID>
where
  T: Send + Sync,
  ID: Send + Sync + Clone + 'static,
{
  type AuditInfo: Send + Sync;

  /// Save with audit information
  async fn save_with_audit(&self, entity: &T, audit: Self::AuditInfo) -> Result<T, Self::Error>;

  /// Update with audit information
  async fn update_with_audit(
    &self,
    id: ID,
    entity: &T,
    audit: Self::AuditInfo,
  ) -> Result<T, Self::Error>;

  /// Delete with audit information
  async fn delete_with_audit(&self, id: ID, audit: Self::AuditInfo) -> Result<bool, Self::Error>;
}

/// Repository trait for soft delete support
#[async_trait]
pub trait SoftDeleteRepository<T, ID>: Repository<T, ID>
where
  T: Send + Sync,
  ID: Send + Sync + Clone + 'static,
{
  /// Soft delete an entity
  async fn soft_delete(&self, id: ID) -> Result<bool, Self::Error>;

  /// Restore a soft deleted entity
  async fn restore(&self, id: ID) -> Result<bool, Self::Error>;

  /// Find all including soft deleted
  async fn find_all_with_deleted(&self) -> Result<Vec<T>, Self::Error>;

  /// Find only soft deleted entities
  async fn find_deleted(&self) -> Result<Vec<T>, Self::Error>;
}

/// Repository trait for transaction support
#[async_trait]
pub trait TransactionalRepository<T, ID>: Repository<T, ID>
where
  T: Send + Sync,
  ID: Send + Sync + Clone + 'static,
{
  type Transaction: Send + Sync;

  /// Begin a new transaction
  async fn begin_transaction(&self) -> Result<Self::Transaction, Self::Error>;

  /// Commit a transaction
  async fn commit_transaction(&self, tx: Self::Transaction) -> Result<(), Self::Error>;

  /// Rollback a transaction
  async fn rollback_transaction(&self, tx: Self::Transaction) -> Result<(), Self::Error>;

  /// Execute a function within a transaction
  async fn with_transaction<F, R>(&self, f: F) -> Result<R, Self::Error>
  where
    F: FnOnce(&Self::Transaction) -> Result<R, Self::Error> + Send,
    R: Send;
}
