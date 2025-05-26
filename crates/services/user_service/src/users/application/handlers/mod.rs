use axum::{extract::State, Json};
use jd_contracts::user::dto::CreateUserRequest;
use jd_core::AppState;

use crate::users::{infrastructure::UserRepositoryImpl, record::UserRecord, CreateUserUseCase};

pub async fn create_user(State(state): State<AppState>, Json(request): Json<CreateUserRequest>) -> Json<UserRecord> {
    let repository = UserRepositoryImpl::new(state);
    let use_case = CreateUserUseCase::new(repository);
    use_case.execute(Json(request)).await.unwrap()
}
