use crate::infra::record::UserRecord;
use axum::{
    extract::{Path, State},
    Json,
};
use jd_core::ModelManager;
use jd_core::{
    base::{rest, rpc, DMC},
    ctx::Ctx,
};
use uuid::Uuid;

use modql::{
    field::Fields,
    filter::{FilterNodes, OpValsString},
};
use serde::Deserialize;

pub mod record;

pub struct UserDmc;

impl DMC for UserDmc {
    const SCHEMA: &'static str = "profile";
    const TABLE: &'static str = "users";
    const ID: &'static str = "user_id";
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct UserFilter {
    pub id: Option<Uuid>,
    pub username: Option<OpValsString>,
}

#[derive(Deserialize, Fields)]
pub struct UpdateUserReq {
    pub id: Uuid,
}
