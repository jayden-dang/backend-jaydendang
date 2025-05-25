use uuid::Uuid;

use super::{EducationLevel, ExperienceLevel, ProfileVisibility, UserGender};

pub struct UserProfile {
    pub user_id: Uuid,
    pub birth_year: Option<i32>,
    pub gender: Option<UserGender>,
    pub occupation: Option<String>,
    pub education_level: Option<EducationLevel>,
    pub experience_level: Option<ExperienceLevel>,
    pub timezone: Option<String>,
    pub country_code: Option<String>,
    pub language: String,
    pub bio: Option<String>,
    pub visibility: ProfileVisibility,
    pub show_progress: bool,
}
