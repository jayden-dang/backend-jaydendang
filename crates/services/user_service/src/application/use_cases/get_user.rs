use jd_contracts::user::dtos::{
    records::user_record::UserRecord, requests::user_filter::UserFilter,
};

use crate::{Result, domain::user_repository_trait::UserRepository};

pub struct GetUserUseCase<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> GetUserUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute_by_username(&self, username: String) -> Result<UserRecord> {
        self.repository
            .find_by_wow(UserFilter {
                email: None,
                username: Some(username.into()),
                is_active: None,
            })
            .await
    }

    pub async fn execute_by_email(&self, email: String) -> Result<UserRecord> {
        self.repository
            .find_by_wow(UserFilter { email: Some(email.into()), username: None, is_active: None })
            .await
    }

    pub async fn execute_by_is_active(&self, is_active: bool) -> Result<UserRecord> {
        self.repository
            .find_by_wow(UserFilter {
                email: None,
                username: None,
                is_active: Some(is_active.into()),
            })
            .await
    }

    pub async fn execute_by_wow(&self, user_filer: UserFilter) -> Result<UserRecord> {
        self.repository.find_by_wow(user_filer).await
    }
}
