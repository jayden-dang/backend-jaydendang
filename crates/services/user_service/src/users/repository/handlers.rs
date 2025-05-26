use crate::{users::record::UserRecord, Result};
use axum::{extract::State, Json};
use jd_contracts::user::dto::CreateUserRequest;
use jd_core::AppState;

use super::{repository_impl::UserRepositoryImpl, repository_trait::UserRepository};

pub async fn create_user(
    State(app_state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserRecord>> {
    let repository = UserRepositoryImpl::new(app_state);
    Ok(Json(repository.create_user(req).await?))
}
