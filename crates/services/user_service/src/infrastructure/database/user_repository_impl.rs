use crate::{
  Error, ProfileDmc, Result, UsersDmc, domain::user_repository_trait::UserRepository,
  error::ErrorMapper,
};
use async_trait::async_trait;
use jd_contracts::user::dtos::{
  records::user_record::UserRecord,
  requests::{
    create_profile_request::CreateUserProfileRequest, create_user_request::CreateUserRequest,
    user_filter::UserFilter,
  },
  responses::create_profile_response::CreateUserProfileResponse,
};
use jd_core::{
  AppState,
  base::{self},
};
use jd_utils::ensure;

pub struct UserRepositoryImpl {
  app_state: AppState,
}

impl UserRepositoryImpl {
  pub fn new(app_state: AppState) -> Self {
    Self { app_state }
  }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
  async fn create(&self, req: CreateUserRequest) -> Result<UserRecord> {
    // Check if user exists with same username or email
    let exists = self.exists(&req).await.unwrap();

    ensure!(!exists, Error::conflict("User with this username or email already exists"));

    base::rest::create::<UsersDmc, _, _>(&self.app_state.mm, req)
      .await
      .map_error()
  }

  async fn create_profile(
    &self,
    request: CreateUserProfileRequest,
  ) -> Result<CreateUserProfileResponse> {
    base::rest::create_with_enum_cast::<ProfileDmc, _, _>(&self.app_state.mm, request)
      .await
      .map_error()
  }

  async fn find_by_wow(&self, req: UserFilter) -> Result<UserRecord> {
    base::rest::get_by_sth::<UsersDmc, _, _>(&self.app_state.mm, Some(req))
      .await
      .map_err(|e| Error::EntityNotFound { entity: e.to_string(), id: 0 })
  }

  async fn exists(&self, req: &CreateUserRequest) -> Result<bool> {
    base::rest::exists::<UsersDmc, _>(
      &self.app_state.mm,
      Some(UserFilter {
        username: Some(req.username.clone().into()),
        email: Some(req.email.clone().into()),
        is_active: None,
      }),
    )
    .await
    .map_error()
  }
}
