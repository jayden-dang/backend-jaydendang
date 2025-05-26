use crate::{
    error::{Error, ErrorMapper},
    users::{record::UserRecord, UsersDmc},
    Result,
};
use async_trait::async_trait;
use jd_contracts::user::dto::{CreateUserRequest, UserFilter};
use jd_core::{
    base::{self},
    AppState,
};
use jd_utils::ensure;
use validator::Validate;

use super::repository_trait::UserRepository;

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
    async fn create_user(&self, req: CreateUserRequest) -> Result<UserRecord> {
        req.validate()?;

        // Check if user exists with same username or email
        let exists = base::rest::exists::<UsersDmc, _>(
            &self.app_state.mm,
            Some(UserFilter {
                username: Some(req.username.clone().into()),
                email: Some(req.email.clone().into()),
                is_active: None,
            }),
        )
        .await
        .map_error()?;

        ensure!(
            !exists,
            Error::conflict("User with this username or email already exists")
        );

        base::rest::create::<UsersDmc, _, _>(&self.app_state.mm, req)
            .await
            .map_error()
    }
}
