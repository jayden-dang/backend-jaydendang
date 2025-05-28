use modql::filter::{FilterNodes, OpValsBool, OpValsString};
use serde::Deserialize;

#[derive(Deserialize, FilterNodes, Default, Debug)]
pub struct UserFilter {
  pub email: Option<OpValsString>,
  pub username: Option<OpValsString>,
  pub is_active: Option<OpValsBool>,
}
