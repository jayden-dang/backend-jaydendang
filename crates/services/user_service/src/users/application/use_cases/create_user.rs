use crate::{
    error::Error,
    users::{domain::repository::UserRepository, record::UserRecord},
    Result,
};
use axum::Json;
use jd_contracts::user::dto::CreateUserRequest;
use validator::Validate;

pub struct CreateUserUseCase<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> CreateUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, Json(request): Json<CreateUserRequest>) -> Result<Json<UserRecord>> {
        request.validate()?;

        self.repository.create(Json(request)).await
    }
}
