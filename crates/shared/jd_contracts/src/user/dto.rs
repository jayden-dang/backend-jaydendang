use modql::{
    field::Fields,
    filter::{FilterNodes, OpValsString},
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize, Fields, FromRow)]
pub struct CreateUserReq {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Serialize, Fields, FromRow)]
pub struct CreateUserRes {
    pub id: Uuid,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Deserialize, FilterNodes, Default, Debug)]
pub struct UserFilter {
    pub email: Option<OpValsString>,
    pub username: Option<OpValsString>,
}
