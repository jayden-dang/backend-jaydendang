use crate::{users::domain::UserRepository, Result};
use axum::{extract::State, Json};
use jd_contracts::user::dto::CreateUserRequest;
use jd_core::AppState;

use crate::users::{infrastructure::UserRepositoryImpl, record::UserRecord, CreateUserUseCase};

pub struct UserHandler<R: UserRepository> {
    pub create_user: CreateUserUseCase<R>,
}

impl<R: UserRepository> UserHandler<R> {
    pub fn new(create_user: CreateUserUseCase<R>) -> Self {
        Self { create_user }
    }

    pub async fn create_user(
        State(state): State<AppState>,
        Json(request): Json<CreateUserRequest>,
    ) -> Result<Json<UserRecord>> {
        let repository = UserRepositoryImpl::new(state);
        let use_case = CreateUserUseCase::new(repository);
        Ok(use_case.execute(Json(request)).await.unwrap())
    }
}
