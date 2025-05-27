// -->>> Region:: START  --->>>  Public Modules
pub mod application;
pub mod infrastructure;
pub mod models;
// <<<-- Region:: END    <<<---  Public Modules

mod domain;
mod error;

use error::Error;
type Result<T> = std::result::Result<T, Error>;

use jd_core::base::DMC;

pub struct UsersDmc;
pub struct ProfileDmc;

impl DMC for UsersDmc {
    const SCHEMA: &'static str = "profile";
    const TABLE: &'static str = "users";
    const ID: &'static str = "user_id";
    const ENUM_COLUMNS: &'static [&'static str] = &[];
}

impl DMC for ProfileDmc {
    const SCHEMA: &'static str = "profile";
    const TABLE: &'static str = "user_profiles";
    const ID: &'static str = "profile_id";
    const ENUM_COLUMNS: &'static [&'static str] =
        &["education_level", "experience_level", "account_status", "profile_visibility"];
}
