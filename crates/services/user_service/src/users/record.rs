use jd_domain::{
    user::{
        user::{Email, HashedPassword, User, Username},
        AccountStatus,
    },
    Id,
};
use jd_utils::time::Rfc3339;
use modql::field::Fields;
use serde::Serialize;
use serde_with::serde_as;
use sqlx::{prelude::FromRow, types::time::OffsetDateTime};

#[serde_as]
#[derive(Serialize, FromRow, Fields, Clone, Debug)]
pub struct UserRecord {
    pub user_id: Id,
    pub email: Email,
    pub username: Username,
    pub password_hash: HashedPassword,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: AccountStatus,
    pub email_verified: bool,
    #[serde_as(as = "Rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde_as(as = "Rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<UserRecord> for User {
    fn from(value: UserRecord) -> Self {
        User {
            user_id: value.user_id,
            email: value.email,
            username: value.username,
            password_hash: value.password_hash,
            first_name: value.first_name,
            last_name: value.last_name,
            status: value.status,
            email_verified: value.email_verified,
        }
    }
}
