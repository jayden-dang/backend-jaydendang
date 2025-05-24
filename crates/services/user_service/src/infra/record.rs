use jd_domain::user::User;
use jd_utils::time::Rfc3339;
use modql::field::Fields;
use serde::Serialize;
use serde_with::serde_as;
use sqlx::{prelude::FromRow, types::time::OffsetDateTime};

#[serde_as]
#[derive(Serialize, FromRow, Fields)]
pub struct UserRecord {
    pub pk_user_id: i64,
    pub email: String,
    pub username: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    #[serde_as(as = "Rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde_as(as = "Rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<UserRecord> for User {
    fn from(value: UserRecord) -> Self {
        User {
            id: value.pk_user_id,
            email: value.email,
            username: value.username,
            first_name: value.first_name,
            last_name: value.last_name,
            is_active: value.is_active,
            email_verified: value.email_verified,
        }
    }
}
