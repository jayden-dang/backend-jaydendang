use jd_domain::{
    user_domain::{
        user::{DomainValidation, Email, HashedPassword, User, Username},
        AccountStatus, EducationLevel, ExperienceLevel, ProfileVisibility, UserGender,
    },
    Id,
};
use jd_utils::{
    regex::USERNAME_REGEX,
    time::{now_utc, Rfc3339},
};
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::{prelude::FromRow, types::time::OffsetDateTime};
use validator::{Validate, ValidationError};

#[serde_as]
#[derive(Serialize, FromRow, Fields, Clone, Debug, Validate, Deserialize)]
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

    pub email_verified: bool,
    #[serde_as(as = "Rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde_as(as = "Rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[serde_as]
#[derive(Serialize, FromRow, Fields, Clone, Debug, Validate)]
pub struct UserProfileRecord {
    pub profile_id: Id, // Add missing profile_id
    pub user_id: Id,
    pub birth_year: Option<i32>,
    pub gender: Option<UserGender>,
    pub occupation: Option<String>,
    pub education_level: Option<EducationLevel>,
    pub experience_level: Option<ExperienceLevel>,
    pub account_status: AccountStatus,
    pub timezone: Option<String>,
    pub country_code: Option<String>,
    pub language_preference: String,
    pub avatar_url: Option<String>,
    #[validate(length(min = 1, max = 1000, message = "Bio must be 1-1000 characters"))]
    pub bio: Option<String>,
    pub profile_visibility: ProfileVisibility, // ✅ Match DB field name
    pub show_progress: bool,
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
            email_verified: value.email_verified,
        };

        user.validate_domain()?;
        Ok(user)
    }
}

// -->>> Region:: START  --->>>  Create User Profile Request
#[derive(Debug, Clone, Serialize, Validate, Deserialize, Fields)]
pub struct CreateUserProfileRequest {
    pub user_id: Id, // FK to users table

    // Demographics (match DB exactly)
    #[validate(range(
        min = 1900,
        max = 2024,
        message = "Birth year must be between 1900 and 2024"
    ))]
    pub birth_year: Option<i32>,

    pub gender: Option<UserGender>,

    #[validate(length(min = 1, max = 100, message = "Occupation must be 1-100 characters"))]
    pub occupation: Option<String>,

    pub education_level: Option<EducationLevel>,
    pub experience_level: Option<ExperienceLevel>,
    pub account_status: Option<AccountStatus>,

    // Location & preferences (match DB field names)
    #[validate(length(min = 1, max = 50, message = "Timezone must be 1-50 characters"))]
    pub timezone: Option<String>,

    #[validate(length(min = 2, max = 2, message = "Country code must be exactly 2 characters"))]
    pub country_code: Option<String>,

    #[validate(length(
        min = 2,
        max = 10,
        message = "Language preference must be 2-10 characters"
    ))]
    pub language_preference: String, // ✅ Match DB: language_preference

    // Profile metadata
    #[validate(url(message = "Invalid avatar URL"))]
    pub avatar_url: Option<String>,

    #[validate(length(min = 1, max = 1000, message = "Bio must be 1-1000 characters"))]
    pub bio: Option<String>,

    // Privacy settings (match DB field names)
    pub profile_visibility: Option<ProfileVisibility>, // ✅ Match DB: profile_visibility
    pub show_progress: Option<bool>,
}

impl CreateUserProfileRequest {
    pub fn new(user_id: Id) -> Self {
        Self {
            user_id,
            birth_year: None,
            gender: None,
            occupation: None,
            education_level: None,
            experience_level: None,
            account_status: None,
            timezone: None,
            country_code: None,
            language_preference: "en".to_string(),
            avatar_url: None,
            bio: None,
            profile_visibility: None, // ✅ Correct field name
            show_progress: None,
        }
    }

    pub fn with_defaults(user_id: Id) -> Self {
        Self {
            user_id,
            birth_year: None,
            gender: None,
            occupation: None,
            education_level: Some(EducationLevel::default()),
            experience_level: Some(ExperienceLevel::default()),
            account_status: Some(AccountStatus::default()),
            timezone: None,
            country_code: None,
            language_preference: "en".to_string(), // ✅ Correct field name
            avatar_url: None,
            bio: None,
            profile_visibility: Some(ProfileVisibility::default()), // ✅ Correct field name
            show_progress: Some(true),
        }
    }
}
// <<<-- Region:: END    <<<---  Create User Profile Request

// -->>> Region:: START  --->>>  Create User Profile Response

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Fields)]
pub struct CreateUserProfileResponse {
    pub profile_id: Id,
    pub user_id: Id,
    pub birth_year: Option<i32>,
    pub gender: Option<UserGender>,
    pub occupation: Option<String>,
    pub education_level: Option<EducationLevel>,
    pub experience_level: Option<ExperienceLevel>,
    pub timezone: Option<String>,
    pub country_code: Option<String>,
    pub account_status: AccountStatus,
    pub language_preference: String, // Changed from language to match DB
    pub bio: Option<String>,
    pub profile_visibility: ProfileVisibility, // Changed from visibility to match DB
    pub show_progress: bool,
    #[serde_as(as = "Rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde_as(as = "Rfc3339")]
    pub updated_at: OffsetDateTime,
}

// Conversion from UserProfileRecord to Response
impl From<UserProfileRecord> for CreateUserProfileResponse {
    fn from(record: UserProfileRecord) -> Self {
        Self {
            profile_id: record.profile_id,
            user_id: record.user_id,
            birth_year: record.birth_year,
            gender: record.gender,
            occupation: record.occupation,
            education_level: record.education_level,
            experience_level: record.experience_level,
            timezone: record.timezone,
            country_code: record.country_code,
            account_status: record.account_status,
            language_preference: record.language_preference,
            bio: record.bio,
            profile_visibility: record.profile_visibility,
            show_progress: record.show_progress,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}
// <<<-- Region:: END    <<<---  Create User Profile Response
