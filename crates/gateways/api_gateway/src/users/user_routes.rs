use axum::{extract::State, Json};
use jd_contracts::user::dto::CreateUserRequest;
use jd_core::AppState;
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
