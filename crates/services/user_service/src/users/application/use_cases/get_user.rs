use crate::{
    error::Error,
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

    pub async fn execute_by_username(&self, username: &str) -> Result<UserRecord> {
        todo!()
    }

    pub async fn execute_by_email(&self, email: &str) -> Result<UserRecord> {
        todo!()
    }
}
