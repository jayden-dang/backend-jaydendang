use std::fmt::Display;

use sea_query::Value;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
mod error;
mod utils;

pub mod user_domain;

pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Id(Uuid);

impl Id {
  pub fn new(id: String) -> Self {
    Self(Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4()))
  }

  pub fn generate() -> Self {
    Self(Uuid::new_v4())
  }

  pub fn value(&self) -> &Uuid {
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
    Value::Uuid(Some(Box::new(id.0)))
  }
}

// SQLx implementations
impl sqlx::Type<sqlx::Postgres> for Id {
  fn type_info() -> sqlx::postgres::PgTypeInfo {
    <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
  }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Id {
  fn decode(
    value: sqlx::postgres::PgValueRef<'r>,
  ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
    let uuid = <Uuid as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
    Ok(Id(uuid))
  }
}

impl<'r> sqlx::Encode<'r, sqlx::Postgres> for Id {
  fn encode_by_ref(
    &self,
    buf: &mut sqlx::postgres::PgArgumentBuffer,
  ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
    <Uuid as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&self.0, buf)
  }
}
