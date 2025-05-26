use axum::Json;
use jd_contracts::user::dto::UserFilter;

use crate::{
    users::{domain::repository::UserRepository, record::UserRecord},
    Result,
};

pub struct GetUserUseCase<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> GetUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, id: &str) -> Result<UserRecord> {
        todo!()
    }

    pub async fn execute_by_username(&self, username: String) -> Result<Json<UserRecord>> {
        self.repository
            .find_by_wow(UserFilter {
                email: None,
                username: Some(username.into()),
                is_active: None,
            })
            .await
    }

    pub async fn execute_by_email(&self, email: String) -> Result<Json<UserRecord>> {
        self.repository
            .find_by_wow(UserFilter { email: Some(email.into()), username: None, is_active: None })
            .await
    }

    pub async fn execute_by_is_active(&self, is_active: bool) -> Result<Json<UserRecord>> {
        self.repository
            .find_by_wow(UserFilter {
                email: None,
                username: None,
                is_active: Some(is_active.into()),
            })
            .await
    }

    pub async fn execute_by_wow(&self, user_filer: UserFilter) -> Result<Json<UserRecord>> {
        self.repository.find_by_wow(user_filer).await
    }
}
