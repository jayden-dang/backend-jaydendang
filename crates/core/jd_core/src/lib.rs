use jd_storage::{dbx::Dbx, new_db_pool};

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
        let dbx = Dbx::new(db_pool, false)?;
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
