use jd_domain::user_domain::{
    AccountStatus, EducationLevel, ExperienceLevel, ProfileVisibility, UserGender,
};
use jd_utils::regex::USERNAME_REGEX;
use modql::{
    field::Fields,
    filter::{FilterNodes, OpValsBool, OpValsString},
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Fields, Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    // Required fields
    #[validate(email(message = "Invalid email format in database record"))]
    pub email: String,

    #[validate(
        length(min = 3, max = 50, message = "Username must be 3-50 characters"),
        regex(path = "USERNAME_REGEX", message = "Username contains invalid characters")
    )]
    pub username: String,
    #[validate(length(min = 8, message = "Password hash too short"))]
    pub password_hash: String,

    // Optional basic info
    #[validate(length(min = 1, max = 100, message = "First name must be 1-100 characters"))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 100, message = "Last name must be 1-100 characters"))]
    pub last_name: Option<String>,
}

#[derive(Deserialize, FilterNodes, Default, Debug)]
pub struct UserFilter {
    pub email: Option<OpValsString>,
    pub username: Option<OpValsString>,
    pub is_active: Option<OpValsBool>,
}

#[derive(Fields, Clone, FromRow, Validate, Serialize)]
pub struct CreateUserProfileRequest {
    pub user_id: Uuid,
    pub birth_year: Option<i32>,
    pub gender: Option<UserGender>,
    pub occupation: Option<String>,
    pub education_level: Option<EducationLevel>,
    pub experience_level: Option<ExperienceLevel>,
    pub timezone: Option<String>,
    pub country_code: Option<String>,
    pub account_status: AccountStatus,
    pub language: String,
    #[validate(length(min = 1, max = 1000, message = "Last name must be 1-100 characters"))]
    pub bio: Option<String>,
    pub visibility: ProfileVisibility,
    pub show_progress: bool,
}
