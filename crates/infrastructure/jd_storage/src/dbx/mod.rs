use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};
use tokio::sync::Mutex;
use tracing::{trace, warn};

use sqlx::{
  IntoArguments, Pool, Postgres, Transaction,
  prelude::FromRow,
  query::{Query, QueryAs},
};

use crate::Db;

mod error;

pub use error::{Error, Result};

/// A database transaction wrapper that supports nested transactions
/// through a counter mechanism.
#[derive(Debug, Clone)]
pub struct Dbx {
  db_pool: Db,
  txn_holder: Arc<Mutex<Option<TxnHolder>>>,
  with_txn: bool,
}

impl Dbx {
  /// Creates a new Dbx instance.
  ///
  /// # Arguments
  ///
  /// * `db_pool` - The database connection pool
  /// * `with_txn` - Whether to enable transaction support
  ///
  /// # Returns
  ///
  /// A new Dbx instance
  pub fn new(db_pool: Db, with_txn: bool) -> Result<Self> {
    Ok(Dbx { db_pool, txn_holder: Arc::default(), with_txn })
  }
}

#[derive(Debug)]
struct TxnHolder {
  txn: Transaction<'static, Postgres>,
  counter: i32,
  is_committed: bool,
}

impl TxnHolder {
  fn new(txn: Transaction<'static, Postgres>) -> Self {
    TxnHolder { txn, counter: 1, is_committed: false }
  }

  fn inc(&mut self) {
    self.counter += 1;
  }

  fn dec(&mut self) -> i32 {
    self.counter -= 1;
    self.counter
  }

  fn is_active(&self) -> bool {
    !self.is_committed && self.counter > 0
  }
}

impl Deref for TxnHolder {
  type Target = Transaction<'static, Postgres>;

  fn deref(&self) -> &Self::Target {
    &self.txn
  }
}

impl DerefMut for TxnHolder {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.txn
  }
}

impl Dbx {
  /// Begins a new transaction or increments the counter if a transaction already exists.
  ///
  /// # Returns
  ///
  /// * `Ok(())` if the transaction was started successfully
  /// * `Err(Error::CannotBeginTxnWithTxnFalse)` if transaction support is disabled
  pub async fn begin_txn(&self) -> Result<()> {
    if !self.with_txn {
      return Err(Error::CannotBeginTxnWithTxnFalse);
    }

    let mut txh_g = self.txn_holder.lock().await;
    // If we already have a tx holder, then, we increment
    if let Some(txh) = txh_g.as_mut() {
      if !txh.is_active() {
        return Err(Error::TxnAlreadyCommitted);
      }
      txh.inc();
      trace!("Transaction counter incremented to {}", txh.counter);
    }
    // If not, we create one with a new transaction
    else {
      let transaction = self.db_pool.begin().await?;
      let _ = txh_g.insert(TxnHolder::new(transaction));
      trace!("New transaction started");
    }

    Ok(())
  }

  /// Rolls back the current transaction if it's the last one in the stack.
  ///
  /// # Returns
  ///
  /// * `Ok(())` if the transaction was rolled back successfully
  /// * `Err(Error::NoTxn)` if no transaction exists
  pub async fn rollback_txn(&self) -> Result<()> {
    let mut txh_g = self.txn_holder.lock().await;
    if let Some(mut txn_holder) = txh_g.take() {
      if txn_holder.counter > 1 {
        txn_holder.counter -= 1;
        trace!("Transaction counter decremented to {}", txn_holder.counter);
        let _ = txh_g.replace(txn_holder);
      } else {
        trace!("Rolling back transaction");
        txn_holder.txn.rollback().await?;
      }
      Ok(())
    } else {
      Err(Error::NoTxn)
    }
  }

  /// Commits the current transaction if it's the last one in the stack.
  ///
  /// # Returns
  ///
  /// * `Ok(())` if the transaction was committed successfully
  /// * `Err(Error::CannotCommitTxnWithTxnFalse)` if transaction support is disabled
  /// * `Err(Error::TxnCantCommitNoOpenTxn)` if no transaction exists
  /// * `Err(Error::TxnAlreadyCommitted)` if the transaction was already committed
  pub async fn commit_txn(&self) -> Result<()> {
    if !self.with_txn {
      return Err(Error::CannotCommitTxnWithTxnFalse);
    }

    let mut txh_g = self.txn_holder.lock().await;
    if let Some(txh) = txh_g.as_mut() {
      if !txh.is_active() {
        return Err(Error::TxnAlreadyCommitted);
      }

      let counter = txh.dec();
      trace!("Transaction counter decremented to {}", counter);

      if counter == 0 {
        if let Some(mut txn) = txh_g.take() {
          trace!("Committing transaction");
          txn.txn.commit().await?;
          txn.is_committed = true;
        } else {
          warn!("Transaction holder was unexpectedly None after counter reached 0");
          return Err(Error::TxnCantCommitNoOpenTxn);
        }
      } else if counter < 0 {
        warn!("Transaction counter went negative: {}", counter);
        return Err(Error::TxnCantCommitNoOpenTxn);
      }

      Ok(())
    } else {
      Err(Error::TxnCantCommitNoOpenTxn)
    }
  }

  /// Returns a reference to the underlying database pool.
  pub fn db(&self) -> &Pool<Postgres> {
    &self.db_pool
  }

  pub async fn fetch_one<'q, O, A>(&self, query: QueryAs<'q, Postgres, O, A>) -> Result<O>
  where
    O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
    A: IntoArguments<'q, Postgres> + 'q,
  {
    let data = if self.with_txn {
      let mut txh_g = self.txn_holder.lock().await;
      if let Some(txn) = txh_g.as_deref_mut() {
        query.fetch_one(txn.as_mut()).await?
      } else {
        query.fetch_one(self.db()).await?
      }
    } else {
      query.fetch_one(self.db()).await?
    };

    Ok(data)
  }

  pub async fn fetch_optional<'q, O, A>(
    &self,
    query: QueryAs<'q, Postgres, O, A>,
  ) -> Result<Option<O>>
  where
    O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
    A: IntoArguments<'q, Postgres> + 'q,
  {
    let data = if self.with_txn {
      let mut txh_g = self.txn_holder.lock().await;
      if let Some(txn) = txh_g.as_deref_mut() {
        query.fetch_optional(txn.as_mut()).await?
      } else {
        query.fetch_optional(self.db()).await?
      }
    } else {
      query.fetch_optional(self.db()).await?
    };

    Ok(data)
  }

  pub async fn fetch_all<'q, O, A>(&self, query: QueryAs<'q, Postgres, O, A>) -> Result<Vec<O>>
  where
    O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
    A: IntoArguments<'q, Postgres> + 'q,
  {
    let data = if self.with_txn {
      let mut txh_g = self.txn_holder.lock().await;
      if let Some(txn) = txh_g.as_deref_mut() {
        query.fetch_all(txn.as_mut()).await?
      } else {
        query.fetch_all(self.db()).await?
      }
    } else {
      query.fetch_all(self.db()).await?
    };

    Ok(data)
  }

  pub async fn execute<'q, A>(&self, query: Query<'q, Postgres, A>) -> Result<u64>
  where
    A: IntoArguments<'q, Postgres> + 'q,
  {
    let row_affected = if self.with_txn {
      let mut txh_g = self.txn_holder.lock().await;
      if let Some(txn) = txh_g.as_deref_mut() {
        query.execute(txn.as_mut()).await?.rows_affected()
      } else {
        query.execute(self.db()).await?.rows_affected()
      }
    } else {
      query.execute(self.db()).await?.rows_affected()
    };

    Ok(row_affected)
  }
}
