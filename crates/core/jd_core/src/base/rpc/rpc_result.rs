use serde::Serialize;

/// A simple RPC result wrapper for data responses
#[derive(Debug, Serialize)]
pub struct DataRpcResult<T> {
  pub data: T,
}

impl<T> From<T> for DataRpcResult<T> {
  fn from(data: T) -> Self {
    Self { data }
  }
}
