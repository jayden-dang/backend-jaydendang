use crate::Result;

use sea_query::Value;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::Id;

use super::AccountStatus;

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: Id,
    pub email: Email,
    pub username: Username,
    pub password_hash: HashedPassword,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: AccountStatus,
    pub email_verified: bool,
}

// -->>> Region:: START  --->>>  Email
#[derive(Debug, Clone, Serialize)]
pub struct Email {
    value: String,
    domain: String,
    local_part: String,
}

impl From<Email> for Value {
    fn from(value: Email) -> Self {
        Value::String(Some(Box::new(value.value)))
    }
}

// <<<-- Region:: END    <<<---  Email

// -->>> Region:: START  --->>>  Username
#[derive(Debug, Clone, Serialize)]
pub struct Username {
    value: String,
}

impl From<Username> for Value {
    fn from(value: Username) -> Self {
        Value::String(Some(Box::new(value.value)))
    }
}
// <<<-- Region:: END    <<<---  Username

// -->>> Region:: START  --->>>  Password
#[derive(Debug, Clone, Serialize)]
pub struct HashedPassword(String);

impl From<HashedPassword> for Value {
    fn from(value: HashedPassword) -> Self {
        Value::String(Some(Box::new(value.0)))
    }
}
// <<<-- Region:: END    <<<---  Password
