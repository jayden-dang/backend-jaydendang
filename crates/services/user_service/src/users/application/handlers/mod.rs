use crate::{
    users::{domain::UserRepository, GetUserUseCase},
    Result,
};
use axum::{
    extract::{Path, State},
    Json,
};
use jd_contracts::user::dto::CreateUserRequest;
use jd_core::AppState;

use crate::users::{infrastructure::UserRepositoryImpl, record::UserRecord, CreateUserUseCase};

pub struct UserHandler<R: UserRepository> {
    pub create_user: CreateUserUseCase<R>,
    pub get_user: GetUserUseCase<R>,
}

impl<R: UserRepository> UserHandler<R> {
    pub fn new(create_user: CreateUserUseCase<R>, get_user: GetUserUseCase<R>) -> Self {
        Self { create_user, get_user }
    }

    pub async fn create_user(
        State(state): State<AppState>,
        Json(request): Json<CreateUserRequest>,
    ) -> Result<Json<UserRecord>> {
        let repository = UserRepositoryImpl::new(state);
        let use_case = CreateUserUseCase::new(repository);
        Ok(use_case.execute(Json(request)).await.unwrap())
    }

    pub async fn get_user_by_username(
        State(state): State<AppState>,
        Path(req): Path<String>,
    ) -> Result<Json<UserRecord>> {
        let repository = UserRepositoryImpl::new(state);
        let use_case = GetUserUseCase::new(repository);
        Ok(use_case.execute_by_username(req).await.unwrap())
    }
}
