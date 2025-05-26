use crate::{
    error::ErrorMapper,
    users::{domain::repository::UserRepository, record::UserRecord, UsersDmc},
    Error, Result,
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
    async fn create(&self, req: CreateUserRequest) -> Result<Json<UserRecord>> {
        // Check if user exists with same username or email
        let exists = self.exists(&req).await.unwrap();

        ensure!(!exists, Error::conflict("User with this username or email already exists"));

        let record = base::rest::create::<UsersDmc, _, _>(&self.app_state.mm, req)
            .await
            .map_error()?;

        Ok(Json(record))
    }

    async fn find_by_wow(&self, req: UserFilter) -> Result<Json<UserRecord>> {
        let record = base::rest::get_by_sth::<UsersDmc, _, _>(&self.app_state.mm, Some(req))
            .await
            .map_err(|e| Error::EntityNotFound { entity: e.to_string(), id: 0 })?;
        Ok(Json(record))
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
