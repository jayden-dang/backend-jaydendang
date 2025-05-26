use crate::{
    users::{domain::repository::UserRepository, record::UserRecord},
    Result,
};
use jd_contracts::user::dto::CreateUserRequest;
use validator::Validate;

pub struct CreateUserUseCase<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> CreateUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, request: CreateUserRequest) -> Result<UserRecord> {
        request.validate()?;
        self.repository.create(request).await
    }
}
