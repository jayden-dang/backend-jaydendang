use jd_domain::{
  Id,
  user_domain::{AccountStatus, EducationLevel, ExperienceLevel, ProfileVisibility, UserGender},
};
use jd_utils::time::Rfc3339;
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::prelude::FromRow;
use time::OffsetDateTime;

use crate::user::dtos::records::user_profile_record::UserProfileRecord;

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
