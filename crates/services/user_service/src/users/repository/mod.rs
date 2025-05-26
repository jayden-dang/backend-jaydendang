use std::sync::Arc;

use crate::{error::{Error, ErrorMapper}, Result};
use axum::{extract::State, Json};
use jd_contracts::user::dto::CreateUserRequest;
use jd_core::{
    base::{self},
    AppState, ModelManager,
};
use validator::Validate;

use super::{record::UserRecord, UsersDmc};

pub async fn create_user(State(mm): State<AppState>, Json(req): Json<CreateUserRequest>) -> Result<Json<UserRecord>> {
    req.validate()?;
    Ok(Json(
        base::rest::create::<UsersDmc, _, _>(&mm.mm, req)
            .await
            .map_error()?
    ))
}

