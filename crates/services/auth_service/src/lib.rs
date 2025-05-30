// -->>> Region:: START  --->>>  Public Modules
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod models;
// <<<-- Region:: END    <<<---  Public Modules

mod error;

pub use error::{Error, Result};

use jd_core::base::DMC;

pub struct AuthNonceDmc;
pub struct AuthUserDmc;

impl DMC for AuthNonceDmc {
  const SCHEMA: &'static str = "auth";
  const TABLE: &'static str = "nonces";
  const ID: &'static str = "address";
  const ENUM_COLUMNS: &'static [&'static str] = &[];
}

impl DMC for AuthUserDmc {
  const SCHEMA: &'static str = "auth";
  const TABLE: &'static str = "users";
  const ID: &'static str = "address";
  const ENUM_COLUMNS: &'static [&'static str] = &[];
}
