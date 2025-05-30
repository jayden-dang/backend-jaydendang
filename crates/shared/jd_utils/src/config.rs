use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize)]
pub struct WebConfig {
  pub addr: String,
}

#[derive(Deserialize)]
pub struct Postgres {
  pub dsn: String,
  pub max_conns: u32,
}

#[derive(Deserialize)]
pub struct Redis {
  pub addr: String,
}

#[derive(Deserialize)]
pub struct SuiConfig {
  pub env: String,
  pub sponsor_address: Option<String>,
  pub sponsor_private_key: Option<String>,
  pub max_gas_budget: Option<u64>,
}

#[derive(Deserialize)]
pub struct Config {
  pub web: WebConfig,
  pub postgres: Postgres,
  pub redis: Redis,
  pub sui: SuiConfig,
  #[serde(rename = "auth_jwt_secret")]
  pub auth_jwt_secret: String,
}

impl Config {
  pub fn from_env() -> crate::Result<Config> {
    config::Config::builder()
      .add_source(config::Environment::default())
      .build()
      .map_err(Error::Config)?
      .try_deserialize::<Config>()
      .map_err(Error::Config)
  }
}
