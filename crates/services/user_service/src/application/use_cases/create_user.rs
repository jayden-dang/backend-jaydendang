use crate::{
    domain::user_repository_trait::UserRepository,
    record::{CreateUserProfileRequest, CreateUserProfileResponse, UserRecord},
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

    pub async fn execute_create_profile(
        &self,
        request: CreateUserProfileRequest,
    ) -> Result<CreateUserProfileResponse> {
        request.validate()?;

        self.repository.create_profile(request).await
    }
}
