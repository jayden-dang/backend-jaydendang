use crate::{users::record::UserRecord, Result};
use async_trait::async_trait;
use axum::Json;
use jd_contracts::user::dto::{CreateUserRequest, UserFilter};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, request: CreateUserRequest) -> Result<Json<UserRecord>>;
    async fn find_by_wow(&self, req: UserFilter) -> Result<Json<UserRecord>>;
    async fn exists(&self, req: &CreateUserRequest) -> Result<bool>;
}
