use crate::Result;
use async_trait::async_trait;
use jd_contracts::user::dtos::{
    records::user_record::UserRecord,
    requests::{
        create_profile_request::CreateUserProfileRequest, create_user_request::CreateUserRequest,
        user_filter::UserFilter,
    },
    responses::create_profile_response::CreateUserProfileResponse,
};

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
