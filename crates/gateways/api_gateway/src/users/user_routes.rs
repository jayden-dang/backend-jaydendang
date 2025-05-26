use axum::extract::{Path, Query};
use axum::{extract::State, Json};
use jd_contracts::user::dto::{CreateUserRequest, UserFilter};
use jd_core::AppState;
use user_service::users::GetUserUseCase;
use user_service::users::{infrastructure::UserRepositoryImpl, record::UserRecord, CreateUserUseCase};
use user_service::Result;

pub async fn create_user_route(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserRecord>> {
    let repository = UserRepositoryImpl::new(state);
    let use_case = CreateUserUseCase::new(repository);
    use_case.execute(Json(request)).await
}

pub async fn get_user_by_username(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<UserRecord>> {
    let repository = UserRepositoryImpl::new(state);
    let use_case = GetUserUseCase::new(repository);
    use_case.execute_by_username(id).await
}

pub async fn get_user_by_email(State(state): State<AppState>, Path(email): Path<String>) -> Result<Json<UserRecord>> {
    let repository = UserRepositoryImpl::new(state);
    let use_case = GetUserUseCase::new(repository);
    use_case.execute_by_email(email).await
}

pub async fn get_user_by_wow(
    State(state): State<AppState>,
    Query(query): Query<UserFilter>,
) -> Result<Json<UserRecord>> {
    let repository = UserRepositoryImpl::new(state);
    let use_case = GetUserUseCase::new(repository);
    use_case.execute_by_wow(query).await
}
