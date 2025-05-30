use std::borrow::Cow;

use jd_storage::dbx;
use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};
use sqlx::error::DatabaseError;

pub type Result<T> = std::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, thiserror::Error)]
#[serde(tag = "type", content = "data")]
pub enum Error {
  #[error("Cannot create model manager provider: {0}")]
  CantCreateModelManagerProvider(String),

  #[error("Configuration error: {0}")]
  Config(#[from] jd_utils::error::Error),

  #[error("List limit exceeded. Maximum: {max}, Requested: {actual}")]
  ListLimitOverMax { max: i64, actual: i64 },

  #[error("Unique constraint violation in table '{table}', constraint '{constraint}'")]
  UniqueViolation { table: String, constraint: String },

  #[error("Count operation failed")]
  CountFail,

  #[error("Database operation failed")]
  Dbx(#[from] dbx::Error),

  #[error("Sea query error: {0}")]
  SeaQuery(
    #[from]
    #[serde_as(as = "DisplayFromStr")]
    sea_query::error::Error,
  ),

  #[error("ModQL conversion error: {0}")]
  ModqlIntoSea(
    #[from]
    #[serde_as(as = "DisplayFromStr")]
    modql::filter::IntoSeaError,
  ),

  #[error("SQL execution error: {0}")]
  Sqlx(
    #[from]
    #[serde_as(as = "DisplayFromStr")]
    sqlx::error::Error,
  ),

  #[error("Entity '{entity}' with id {id} not found")]
  EntityNotFound { entity: &'static str, id: i64 },

  #[error("Redis operation failed: {0}")]
  Redis(
    #[from]
    #[serde_as(as = "DisplayFromStr")]
    redis::RedisError,
  ),

  #[error("Sui SDK error: {0}")]
  SuiSdk(
    #[from]
    #[serde_as(as = "DisplayFromStr")]
    sui_sdk::error::Error,
  ),

  #[error("Invalid enum value: {value}")]
  InvalidEnumValue { value: String },

  #[error("Column not found: {column}")]
  ColumnNotFound { column: String },

  #[error("Failed to create Sui client: {0}")]
  CantCreateSuiClient(String),

  #[error("RPC error: {0}")]
  RpcError(String),
}

impl Error {
  // -- Constructor methods for better ergonomics
  pub fn cant_create_model_manager(reason: impl Into<String>) -> Self {
    Self::CantCreateModelManagerProvider(reason.into())
  }

  pub fn list_limit_exceeded(max: i64, actual: i64) -> Self {
    Self::ListLimitOverMax { max, actual }
  }

  pub fn unique_violation(table: impl Into<String>, constraint: impl Into<String>) -> Self {
    Self::UniqueViolation { table: table.into(), constraint: constraint.into() }
  }

  pub fn count_fail() -> Self {
    Self::CountFail
  }

  pub fn entity_not_found(entity: &'static str, id: i64) -> Self {
    Self::EntityNotFound { entity, id }
  }

  // -- Error analysis methods
  pub fn is_unique_violation(&self) -> bool {
    matches!(self, Self::UniqueViolation { .. })
      || self
        .as_database_error()
        .and_then(|db_err| db_err.code())
        .map(|code| code == "23505")
        .unwrap_or(false)
  }

  pub fn is_not_found(&self) -> bool {
    matches!(self, Self::EntityNotFound { .. })
  }

  pub fn is_database_error(&self) -> bool {
    matches!(self, Self::Dbx(_) | Self::Sqlx(_))
  }

  pub fn is_validation_error(&self) -> bool {
    matches!(self, Self::ListLimitOverMax { .. })
  }

  /// This function will transform the error into a more precise variant if it is an SQLX or PGError Unique Violation.
  /// The resolver can contain a function (table_name: &str, constraint: &str) that may return a specific Error if desired.
  /// If the resolver is None, or if the resolver function returns None, it will default to Error::UniqueViolation {table, constraint}.
  pub fn resolve_unique_violation<F>(self, resolver: Option<F>) -> Self
  where
    F: FnOnce(&str, &str) -> Option<Self>,
  {
    match self
      .as_database_error()
      .map(|db_error| (db_error.code(), db_error.table(), db_error.constraint()))
    {
      // "23505" => postgresql "unique violation"
      Some((Some(Cow::Borrowed("23505")), Some(table), Some(constraint))) => resolver
        .and_then(|fun| fun(table, constraint))
        .unwrap_or_else(|| Error::UniqueViolation {
          table: table.to_string(),
          constraint: constraint.to_string(),
        }),
      _ => self,
    }
  }

  /// A convenient function to return the eventual database error (Postgres)
  /// if this Error is an SQLX Error that contains a database error.
  pub fn as_database_error(&self) -> Option<&(dyn DatabaseError + 'static)> {
    match self {
      Error::Dbx(dbx::Error::Sqlx(sqlx_error)) => sqlx_error.as_database_error(),
      Error::Sqlx(sqlx_error) => sqlx_error.as_database_error(),
      _ => None,
    }
  }

  /// Extract constraint name from unique violation error
  pub fn constraint_name(&self) -> Option<&str> {
    match self {
      Self::UniqueViolation { constraint, .. } => Some(constraint),
      _ => self.as_database_error()?.constraint(),
    }
  }

  /// Extract table name from database error
  pub fn table_name(&self) -> Option<&str> {
    match self {
      Self::UniqueViolation { table, .. } => Some(table),
      _ => self.as_database_error()?.table(),
    }
  }
}
