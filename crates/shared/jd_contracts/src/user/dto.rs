use modql::field::Fields;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Fields)]
pub struct CreateUserReq {
    pub pk_user_id: i64,
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Serialize)]
pub struct CreateUserRes {
    pub user_id: i64,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
