use crate::Result;
use async_trait::async_trait;
use jd_contracts::user::dtos::{
  records::user_record::UserRecord, requests::create_user_request::CreateUserRequest,
};

#[async_trait]
#[allow(unused)]
pub trait UserService: Send + Sync {
  async fn create_user(&self, request: CreateUserRequest) -> Result<UserRecord>;
  async fn get_user(&self, id: &str) -> Result<UserRecord>;
  async fn get_user_by_username(&self, username: &str) -> Result<UserRecord>;
  async fn get_user_by_email(&self, email: &str) -> Result<UserRecord>;
}
