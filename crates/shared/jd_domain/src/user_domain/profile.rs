use crate::Id;

use super::{AccountStatus, EducationLevel, ExperienceLevel, ProfileVisibility, UserGender};

pub struct UserProfile {
  pub user_id: Id,
  pub birth_year: Option<i32>,
  pub gender: Option<UserGender>,
  pub occupation: Option<String>,
  pub education_level: Option<EducationLevel>,
  pub experience_level: Option<ExperienceLevel>,
  pub timezone: Option<String>,
  pub country_code: Option<String>,
  pub account_status: AccountStatus,
  pub language: String,
  pub bio: Option<String>,
  pub visibility: ProfileVisibility,
  pub show_progress: bool,
}
