use std::fmt::Display;

use sea_query::Value;
use serde::Serialize;
use uuid::Uuid;
mod error;
mod utils;

pub mod user;

pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Clone, Serialize)]
pub struct Id(String);

impl Id {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// sea-query implementations
impl From<Id> for Value {
    fn from(id: Id) -> Self {
        Value::String(Some(Box::new(id.0)))
    }
}

// SQLx implementations
impl sqlx::Type<sqlx::Postgres> for Id {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}
