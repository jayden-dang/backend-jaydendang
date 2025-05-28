use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "type", content = "data")]
pub enum Error {
  // -- Transaction State Errors
  #[error("Cannot commit transaction: no open transaction found")]
  TxnCantCommitNoOpenTxn,

  #[error("Cannot begin transaction: transaction flag is disabled")]
  CannotBeginTxnWithTxnFalse,

  #[error("Cannot commit transaction: transaction flag is disabled")]
  CannotCommitTxnWithTxnFalse,

  #[error("No active transaction available")]
  NoTxn,

  #[error("Transaction has already been committed")]
  TxnAlreadyCommitted,

  #[error("Transaction has already been rolled back")]
  TxnAlreadyRolledBack,

  #[error("Transaction timeout after {timeout_ms}ms")]
  TxnTimeout { timeout_ms: u64 },

  #[error("Transaction deadlock detected")]
  TxnDeadlock,

  #[error("Transaction isolation level conflict")]
  TxnIsolationConflict,

  #[error("Transaction savepoint '{savepoint}' not found")]
  SavepointNotFound { savepoint: String },

  #[error("Cannot create savepoint in non-transactional context")]
  SavepointWithoutTransaction,

  #[error("Nested transaction not supported")]
  NestedTransactionNotSupported,

  #[error("Transaction lock wait timeout")]
  TxnLockTimeout,

  #[error("Connection lost during transaction")]
  TxnConnectionLost,

  // -- External Dependencies
  #[error("Database error: {0}")]
  Sqlx(
    #[from]
    #[serde_as(as = "DisplayFromStr")]
    sqlx::Error,
  ),

  #[error("Migration error: {0}")]
  Migration(
    #[from]
    #[serde_as(as = "DisplayFromStr")]
    sqlx::migrate::MigrateError,
  ),
}

// ============================================================================
// Error Classification & Analysis
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
  Low,      // Expected transaction state errors
  Medium,   // Timeout, deadlock issues
  High,     // Connection issues, lock timeouts
  Critical, // Data corruption, migration failures
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCategory {
  TransactionState,
  TransactionControl,
  DatabaseConnection,
  Concurrency,
  Configuration,
}

impl Error {
  // -- Constructor methods for better ergonomics
  pub fn txn_cant_commit() -> Self {
    Self::TxnCantCommitNoOpenTxn
  }

  pub fn cannot_begin_txn() -> Self {
    Self::CannotBeginTxnWithTxnFalse
  }

  pub fn cannot_commit_txn() -> Self {
    Self::CannotCommitTxnWithTxnFalse
  }

  pub fn no_txn() -> Self {
    Self::NoTxn
  }

  pub fn already_committed() -> Self {
    Self::TxnAlreadyCommitted
  }

  pub fn already_rolled_back() -> Self {
    Self::TxnAlreadyRolledBack
  }

  pub fn timeout(timeout_ms: u64) -> Self {
    Self::TxnTimeout { timeout_ms }
  }

  pub fn deadlock() -> Self {
    Self::TxnDeadlock
  }

  pub fn isolation_conflict() -> Self {
    Self::TxnIsolationConflict
  }

  pub fn savepoint_not_found(savepoint: impl Into<String>) -> Self {
    Self::SavepointNotFound { savepoint: savepoint.into() }
  }

  pub fn savepoint_without_transaction() -> Self {
    Self::SavepointWithoutTransaction
  }

  pub fn nested_not_supported() -> Self {
    Self::NestedTransactionNotSupported
  }

  pub fn lock_timeout() -> Self {
    Self::TxnLockTimeout
  }

  pub fn connection_lost() -> Self {
    Self::TxnConnectionLost
  }

  // -- Error analysis methods
  pub fn severity(&self) -> ErrorSeverity {
    match self {
      // Low severity - expected transaction state errors
      Self::TxnCantCommitNoOpenTxn
      | Self::CannotBeginTxnWithTxnFalse
      | Self::CannotCommitTxnWithTxnFalse
      | Self::NoTxn
      | Self::TxnAlreadyCommitted
      | Self::TxnAlreadyRolledBack
      | Self::SavepointNotFound { .. }
      | Self::SavepointWithoutTransaction
      | Self::NestedTransactionNotSupported => ErrorSeverity::Low,

      // Medium severity - concurrency issues
      Self::TxnTimeout { .. }
      | Self::TxnDeadlock
      | Self::TxnIsolationConflict
      | Self::TxnLockTimeout => ErrorSeverity::Medium,

      // High severity - connection/infrastructure issues
      Self::TxnConnectionLost => ErrorSeverity::High,

      // Critical severity - data integrity issues
      Self::Migration(_) => ErrorSeverity::Critical,

      // Variable severity based on SQLx error type
      Self::Sqlx(sqlx_err) => {
        if sqlx_err.as_database_error().is_some() {
          ErrorSeverity::Medium
        } else {
          ErrorSeverity::High
        }
      }
    }
  }

  pub fn category(&self) -> ErrorCategory {
    match self {
      Self::TxnCantCommitNoOpenTxn
      | Self::TxnAlreadyCommitted
      | Self::TxnAlreadyRolledBack
      | Self::NoTxn => ErrorCategory::TransactionState,

      Self::CannotBeginTxnWithTxnFalse
      | Self::CannotCommitTxnWithTxnFalse
      | Self::SavepointNotFound { .. }
      | Self::SavepointWithoutTransaction
      | Self::NestedTransactionNotSupported => ErrorCategory::TransactionControl,

      Self::TxnConnectionLost => ErrorCategory::DatabaseConnection,

      Self::TxnTimeout { .. }
      | Self::TxnDeadlock
      | Self::TxnIsolationConflict
      | Self::TxnLockTimeout => ErrorCategory::Concurrency,

      Self::Sqlx(_) => ErrorCategory::DatabaseConnection,

      Self::Migration(_) => ErrorCategory::Configuration,
    }
  }

  pub fn is_retryable(&self) -> bool {
    matches!(
      self,
      Self::TxnDeadlock | Self::TxnTimeout { .. } | Self::TxnLockTimeout | Self::TxnConnectionLost
    )
  }

  pub fn is_transaction_state_error(&self) -> bool {
    matches!(self.category(), ErrorCategory::TransactionState | ErrorCategory::TransactionControl)
  }

  pub fn is_concurrency_error(&self) -> bool {
    matches!(self.category(), ErrorCategory::Concurrency)
  }

  pub fn is_connection_error(&self) -> bool {
    matches!(self.category(), ErrorCategory::DatabaseConnection)
  }

  pub fn requires_rollback(&self) -> bool {
    matches!(
      self,
      Self::TxnDeadlock
        | Self::TxnTimeout { .. }
        | Self::TxnIsolationConflict
        | Self::TxnConnectionLost
    )
  }

  // -- SQLx error analysis
  pub fn is_unique_violation(&self) -> bool {
    match self {
      Self::Sqlx(sqlx_err) => {
        sqlx_err
          .as_database_error()
          .and_then(|db_err| db_err.code())
          .map(|code| code == "23505") // PostgreSQL unique violation
          .unwrap_or(false)
      }
      _ => false,
    }
  }

  pub fn is_foreign_key_violation(&self) -> bool {
    match self {
      Self::Sqlx(sqlx_err) => {
        sqlx_err
          .as_database_error()
          .and_then(|db_err| db_err.code())
          .map(|code| code == "23503") // PostgreSQL foreign key violation
          .unwrap_or(false)
      }
      _ => false,
    }
  }

  pub fn is_serialization_failure(&self) -> bool {
    match self {
      Self::Sqlx(sqlx_err) => {
        sqlx_err
          .as_database_error()
          .and_then(|db_err| db_err.code())
          .map(|code| code == "40001") // PostgreSQL serialization failure
          .unwrap_or(false)
      }
      _ => false,
    }
  }

  // -- Retry strategy recommendations
  pub fn recommended_retry_delay_ms(&self) -> Option<u64> {
    match self {
      Self::TxnDeadlock => Some(100),       // Short delay for deadlocks
      Self::TxnTimeout { .. } => Some(500), // Medium delay for timeouts
      Self::TxnLockTimeout => Some(200),
      Self::TxnConnectionLost => Some(1000), // Longer delay for connection issues
      _ => None,
    }
  }

  pub fn max_retry_attempts(&self) -> Option<u32> {
    match self {
      Self::TxnDeadlock => Some(5), // More retries for deadlocks
      Self::TxnTimeout { .. } => Some(3),
      Self::TxnLockTimeout => Some(3),
      Self::TxnConnectionLost => Some(2),
      _ => None,
    }
  }
}
