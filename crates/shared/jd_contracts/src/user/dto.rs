use jd_domain::Id;
use jd_utils::regex::USERNAME_REGEX;
use modql::{
    field::Fields,
    filter::{FilterNodes, OpValsBool, OpValsString},
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use validator::Validate;

#[derive(Fields, Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    // Required fields
    #[validate(email(message = "Invalid email format in database record"))]
    pub email: String,

    #[validate(
        length(min = 3, max = 50, message = "Username must be 3-50 characters"),
        regex(path = "USERNAME_REGEX", message = "Username contains invalid characters")
    )]
    pub username: String,
    #[validate(length(min = 8, message = "Password hash too short"))]
    pub password_hash: String,

    // Optional basic info
    #[validate(length(min = 1, max = 100, message = "First name must be 1-100 characters"))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 100, message = "Last name must be 1-100 characters"))]
    pub last_name: Option<String>,
}

#[derive(Deserialize, FilterNodes, Default, Debug)]
pub struct UserFilter {
    pub email: Option<OpValsString>,
    pub username: Option<OpValsString>,
    pub is_active: Option<OpValsBool>,
}
