use modql::{
    field::Fields,
    filter::{FilterNodes, OpValsBool, OpValsString},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Fields, Debug, Deserialize)]
pub struct CreateUserRequest {
    // Required fields
    pub email: String,
    pub username: String,
    pub password_hash: String,

    // Optional basic info
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile_created: bool,
    pub acquisition_tracked: bool,
    pub message: String,
}

#[derive(Deserialize, FilterNodes, Default, Debug)]
pub struct UserFilter {
    pub email: Option<OpValsString>,
    pub username: Option<OpValsString>,
    pub is_active: Option<OpValsBool>,
}
