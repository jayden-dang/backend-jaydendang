use crate::config::Config;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> sqlx::Result<Db> {
    let cfg = Config::from_env().expect("Cannot load env");

    PgPoolOptions::new()
        .max_connections(cfg.postgres.max_conns)
        .connect(&cfg.postgres.dsn)
        .await
}
