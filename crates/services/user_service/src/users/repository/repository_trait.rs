use crate::{users::record::UserRecord, Result};
use async_trait::async_trait;
use jd_contracts::user::dto::CreateUserRequest;

#[async_trait]
pub trait UserRepository {
    async fn create_user(&self, req: CreateUserRequest) -> Result<UserRecord>;
}
