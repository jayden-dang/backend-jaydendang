use crate::{Result, domain::user_repository_trait::UserRepository};
use jd_contracts::user::dtos::{
  records::user_record::UserRecord,
  requests::{
    create_profile_request::CreateUserProfileRequest, create_user_request::CreateUserRequest,
  },
  responses::create_profile_response::CreateUserProfileResponse,
};
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
