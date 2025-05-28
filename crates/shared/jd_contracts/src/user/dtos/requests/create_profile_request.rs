use jd_domain::{
    Id,
    user_domain::{AccountStatus, EducationLevel, ExperienceLevel, ProfileVisibility, UserGender},
};
use modql::field::Fields;
use serde::{Deserialize, Serialize};
use validator::Validate;

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
