use std::sync::Arc;

use crate::{error::Error, Result};
use axum::{extract::State, Json};
use jd_contracts::user::dto::CreateUserRequest;
use jd_core::{base, ModelManager};

use super::{record::UserRecord, UsersDmc};

pub async fn create_user(
    State(mm): State<ModelManager>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserRecord>> {
    Ok(Json(
        base::rest::create::<UsersDmc, _, _>(&mm, req)
            .await
            .map_err(|e| Error::CoreError(Arc::new(e)))?,
    ))
}
