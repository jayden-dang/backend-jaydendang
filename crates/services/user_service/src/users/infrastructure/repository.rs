use crate::{
    users::{domain::repository::UserRepository, record::UserRecord, UsersDmc},
    Error, Result, error::ErrorMapper,
};
use async_trait::async_trait;
use axum::Json;
use jd_contracts::user::dto::{CreateUserRequest, UserFilter};
use jd_core::{
    base::{self},
    AppState,
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
    async fn create(&self, Json(req): Json<CreateUserRequest>) -> Result<Json<UserRecord>> {
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

        let record = base::rest::create::<UsersDmc, _, _>(&self.app_state.mm, req)
            .await
            .map_error()?;

        Ok(Json(record))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<UserRecord>> {
        todo!()
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<UserRecord>> {
        todo!()
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<UserRecord>> {
        todo!()
    }

    async fn exists(&self, username: &str, email: &str) -> Result<bool> {
        todo!()
    }
}
