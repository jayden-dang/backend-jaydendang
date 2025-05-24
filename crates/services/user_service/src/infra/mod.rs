use crate::infra::record::UserRecord;
use jd_contracts::user::dto::{CreateUserReq, CreateUserRes};
use jd_core::base;
use jd_core::ctx::Ctx;
use jd_core::ModelManager;
use jd_core::Result;
use jd_core::{base::DMC, generate_common_bmc_fns};
use modql::filter::ListOptions;

use modql::{
    field::Fields,
    filter::{FilterNodes, OpValsString},
};
use serde::Deserialize;

pub(crate) mod record;

pub struct UserDmc;

impl DMC for UserDmc {
    const SCHEMA: &'static str = "user";
    const TABLE: &'static str = "profile";
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct UserFilter {
    pub id: Option<i64>,
    pub username: Option<OpValsString>,
}

#[derive(Deserialize, Fields)]
pub struct UpdateUserReq {
    pub id: i64,
}

generate_common_bmc_fns!(
    DMC: UserDmc,
    Entity: UserRecord,
    ReqCreate: CreateUserReq,
    ResCreate: CreateUserRes,
    ReqUpdate: UpdateUserReq,
    Filter: UserFilter,
);
