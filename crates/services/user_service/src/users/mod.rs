pub(crate) mod record;
pub mod repository;

use jd_core::base::DMC;

pub struct UsersDmc;

impl DMC for UsersDmc {
    const SCHEMA: &'static str = "profile";
    const TABLE: &'static str = "users";
    const ID: &'static str = "user_id";
}
