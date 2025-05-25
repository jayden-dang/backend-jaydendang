use std::sync::Arc;

use crate::{error::Error, Result};
use axum::{extract::State, Json};
use jd_contracts::user::dto::CreateUserRequest;
use jd_core::{
    base::{self},
    AppState, ModelManager,
};

use super::{record::UserRecord, UsersDmc};

pub async fn create_user(State(mm): State<AppState>, Json(req): Json<CreateUserRequest>) -> Result<Json<UserRecord>> {
    Ok(Json(
        base::rest::create::<UsersDmc, _, _>(&mm.mm, req)
            .await
            .map_err(|e| {
                let error_str = e.as_ref().to_string();
                if error_str.contains("23505") && error_str.contains("users_email_key") {
                    Error::conflict("Email already exists")
                } else {
                    Error::CoreError(Arc::new(e))
                }
            })?,
    ))
}

