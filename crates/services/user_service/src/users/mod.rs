pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod record;

use jd_core::base::DMC;

pub struct UsersDmc;

impl DMC for UsersDmc {
    const SCHEMA: &'static str = "profile";
    const TABLE: &'static str = "users";
    const ID: &'static str = "user_id";
}

pub use application::use_cases::*;
