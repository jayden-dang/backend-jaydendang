use crate::base::DBbmc;

pub(crate) mod record;

pub struct UserBmc;

impl DBbmc for UserBmc {
    const SCHEMA: &'static str = "user";
    const TABLE: &'static str = "profile";
}

impl UserBmc {}
