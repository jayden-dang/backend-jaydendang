use crate::{users::record::UserRecord, Result};
use async_trait::async_trait;
use axum::Json;
use jd_contracts::user::dto::CreateUserRequest;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, Json(request): Json<CreateUserRequest>) -> Result<Json<UserRecord>>;
    async fn find_by_id(&self, id: &str) -> Result<Option<UserRecord>>;
    async fn find_by_username(&self, username: &str) -> Result<Option<UserRecord>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<UserRecord>>;
    async fn exists(&self, username: &str, email: &str) -> Result<bool>;
}
