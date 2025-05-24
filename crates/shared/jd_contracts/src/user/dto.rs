use modql::field::Fields;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Deserialize, Fields, FromRow)]
pub struct CreateUserReq {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Serialize, Fields, FromRow)]
pub struct CreateUserRes {
    pub id: i64,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}
