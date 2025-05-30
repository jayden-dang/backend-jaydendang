use modql::filter::ListOptions;
use rpc_router::{IntoDefaultRpcParams, IntoParams};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_with::{OneOrMany, serde_as};

/// Params structure for any RPC Create call.
#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
  pub data: D,
}

impl<D> IntoParams for ParamsForCreate<D> where D: DeserializeOwned + Send {}

/// Params structure for any RPC Update call.
#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
  pub id: i64,
  pub data: D,
}

impl<D> IntoParams for ParamsForUpdate<D> where D: DeserializeOwned + Send {}

/// Params structure for any RPC Update call.
#[derive(Deserialize)]
pub struct ParamsIded {
  pub id: i64,
}
impl IntoParams for ParamsIded {}

/// Params structure for any RPC List call.
#[serde_as]
#[derive(Deserialize, Default)]
pub struct ParamsList<F>
where
  F: DeserializeOwned,
{
  #[serde_as(deserialize_as = "Option<OneOrMany<_>>")]
  pub filters: Option<Vec<F>>,
  pub list_options: Option<ListOptions>,
}

impl<D> IntoDefaultRpcParams for ParamsList<D> where D: DeserializeOwned + Send + Default {}
