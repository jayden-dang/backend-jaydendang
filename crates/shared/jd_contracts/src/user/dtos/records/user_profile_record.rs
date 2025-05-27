use jd_domain::{
    user_domain::{AccountStatus, EducationLevel, ExperienceLevel, ProfileVisibility, UserGender},
    Id,
};
use jd_utils::time::Rfc3339;
use modql::field::Fields;
use serde::Serialize;
use serde_with::serde_as;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;
use validator::Validate;

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
