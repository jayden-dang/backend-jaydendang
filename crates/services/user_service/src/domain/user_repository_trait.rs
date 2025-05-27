use crate::{
    record::{CreateUserProfileRequest, CreateUserProfileResponse, UserRecord},
    Result,
};
use async_trait::async_trait;
use jd_contracts::user::dto::{CreateUserRequest, UserFilter};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, request: CreateUserRequest) -> Result<UserRecord>;
    async fn create_profile(
        &self,
        request: CreateUserProfileRequest,
    ) -> Result<CreateUserProfileResponse>;
    async fn find_by_wow(&self, req: UserFilter) -> Result<UserRecord>;
    async fn exists(&self, req: &CreateUserRequest) -> Result<bool>;
}
