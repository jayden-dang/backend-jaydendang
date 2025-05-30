pub mod dbx;
pub mod repository;
pub mod utils;

use jd_utils::config::Config;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> sqlx::Result<Db> {
  let cfg = Config::from_env().expect("Cannot load env");

  PgPoolOptions::new()
    .max_connections(cfg.postgres.max_conns)
    .connect(&cfg.postgres.dsn)
    .await
}
