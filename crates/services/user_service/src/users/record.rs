use jd_domain::{
    user::{
        user::{DomainValidation, Email, HashedPassword, User, Username},
        AccountStatus,
    },
    Id,
};
use jd_utils::{
    regex::USERNAME_REGEX,
    time::{now_utc, Rfc3339},
};
use modql::field::Fields;
use serde::Serialize;
use serde_with::serde_as;
use sqlx::{prelude::FromRow, types::time::OffsetDateTime};
use validator::{Validate, ValidationError};

#[serde_as]
#[derive(Serialize, FromRow, Fields, Clone, Debug, Validate)]
pub struct UserRecord {
    pub user_id: Id,
    #[validate(email(message = "Invalid email format in database record"))]
    pub email: String,
    #[validate(
        length(min = 3, max = 50, message = "Username must be 3-50 characters"),
        regex(path = "USERNAME_REGEX", message = "Username contains invalid characters")
    )]
    pub username: String,

    #[validate(length(min = 8, message = "Password hash too short"))]
    pub password_hash: String,

    #[validate(length(min = 1, max = 100, message = "First name must be 1-100 characters"))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 100, message = "Last name must be 1-100 characters"))]
    pub last_name: Option<String>,

    // pub status: AccountStatus,
    pub email_verified: bool,
    #[serde_as(as = "Rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde_as(as = "Rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<User> for UserRecord {
    fn from(value: User) -> Self {
        UserRecord {
            user_id: value.user_id,
            email: value.email.value,
            username: value.username.value,
            password_hash: value.password_hash.value,
            first_name: value.first_name,
            last_name: value.last_name,
            // status: value.status,
            email_verified: value.email_verified,
            created_at: now_utc(),
            updated_at: now_utc(),
        }
    }
}

impl TryFrom<UserRecord> for User {
    type Error = ValidationError;

    fn try_from(value: UserRecord) -> Result<Self, Self::Error> {
        let email = Email::new(value.email)?;
        let username = Username::new(value.username)?;
        let password_hash = HashedPassword::new(value.password_hash);

        let user = User {
            user_id: value.user_id,
            email,
            username,
            password_hash,
            first_name: value.first_name,
            last_name: value.last_name,
            // status: value.status,
            email_verified: value.email_verified,
        };

        user.validate_domain()?;
        Ok(user)
    }
}
