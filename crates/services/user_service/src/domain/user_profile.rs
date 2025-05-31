use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

// Import enums from the SQL schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "experience_level", rename_all = "snake_case")]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "subscription_tier", rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Premium,
    Enterprise,
    Lifetime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "education_level", rename_all = "snake_case")]
pub enum EducationLevel {
    HighSchool,
    Bachelor,
    Master,
    Phd,
    Bootcamp,
    SelfTaught,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "profile_visibility", rename_all = "lowercase")]
pub enum ProfileVisibility {
    Public,
    Private,
    Friends,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_gender", rename_all = "snake_case")]
pub enum UserGender {
    Male,
    Female,
    NonBinary,
    PreferNotToSay,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "registration_source", rename_all = "lowercase")]
pub enum RegistrationSource {
    Organic,
    Google,
    Facebook,
    Twitter,
    Referral,
    PaidAd,
    Blog,
    Youtube,
    Email,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Validate)]
pub struct UserProfile {
    pub profile_id: Uuid,
    pub user_id: Uuid,
    
    // Personal information
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,
    #[validate(length(max = 1000))]
    pub bio: Option<String>,
    #[validate(url)]
    pub avatar_url: Option<String>,
    
    // Demographics
    #[validate(range(min = 1900, max = 2020))]
    pub birth_year: Option<i32>,
    pub gender: Option<UserGender>,
    #[validate(length(max = 100))]
    pub occupation: Option<String>,
    pub education_level: Option<EducationLevel>,
    pub experience_level: Option<ExperienceLevel>,
    
    // Location & preferences
    #[validate(length(max = 50))]
    pub timezone: Option<String>,
    #[validate(length(min = 2, max = 2))]
    pub country_code: Option<String>,
    #[validate(length(max = 10))]
    pub language_preference: String,
    
    // Privacy settings
    pub profile_visibility: ProfileVisibility,
    pub show_activity: bool,
    pub show_email: bool,
    
    // Subscription & account
    pub subscription_tier: SubscriptionTier,
    pub registration_source: RegistrationSource,
    
    // Timestamps
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserProfileForCreate {
    pub user_id: Uuid,
    
    // Personal information
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,
    #[validate(length(max = 1000))]
    pub bio: Option<String>,
    #[validate(url)]
    pub avatar_url: Option<String>,
    
    // Demographics
    #[validate(range(min = 1900, max = 2020))]
    pub birth_year: Option<i32>,
    pub gender: Option<UserGender>,
    #[validate(length(max = 100))]
    pub occupation: Option<String>,
    pub education_level: Option<EducationLevel>,
    pub experience_level: Option<ExperienceLevel>,
    
    // Location & preferences
    #[validate(length(max = 50))]
    pub timezone: Option<String>,
    #[validate(length(min = 2, max = 2))]
    pub country_code: Option<String>,
    #[validate(length(max = 10))]
    pub language_preference: Option<String>,
    
    // Privacy settings
    pub profile_visibility: Option<ProfileVisibility>,
    pub show_activity: Option<bool>,
    pub show_email: Option<bool>,
    
    // Subscription & account
    pub subscription_tier: Option<SubscriptionTier>,
    pub registration_source: Option<RegistrationSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserProfileForUpdate {
    // Personal information
    #[validate(length(min = 1, max = 100))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub last_name: Option<String>,
    #[validate(length(max = 1000))]
    pub bio: Option<String>,
    #[validate(url)]
    pub avatar_url: Option<String>,
    
    // Demographics
    #[validate(range(min = 1900, max = 2020))]
    pub birth_year: Option<i32>,
    pub gender: Option<UserGender>,
    #[validate(length(max = 100))]
    pub occupation: Option<String>,
    pub education_level: Option<EducationLevel>,
    pub experience_level: Option<ExperienceLevel>,
    
    // Location & preferences
    #[validate(length(max = 50))]
    pub timezone: Option<String>,
    #[validate(length(min = 2, max = 2))]
    pub country_code: Option<String>,
    #[validate(length(max = 10))]
    pub language_preference: Option<String>,
    
    // Privacy settings
    pub profile_visibility: Option<ProfileVisibility>,
    pub show_activity: Option<bool>,
    pub show_email: Option<bool>,
    
    // Subscription & account
    pub subscription_tier: Option<SubscriptionTier>,
}

impl Default for ProfileVisibility {
    fn default() -> Self {
        ProfileVisibility::Public
    }
}

impl Default for SubscriptionTier {
    fn default() -> Self {
        SubscriptionTier::Free
    }
}

impl Default for RegistrationSource {
    fn default() -> Self {
        RegistrationSource::Organic
    }
}

impl UserProfile {
    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.first_name.is_some() && 
        self.last_name.is_some() && 
        self.experience_level.is_some()
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => format!("{} {}", first, last),
            (Some(first), None) => first.clone(),
            (None, Some(last)) => last.clone(),
            (None, None) => "Anonymous User".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn is_premium(&self) -> bool {
        matches!(
            self.subscription_tier,
            SubscriptionTier::Premium | SubscriptionTier::Enterprise | SubscriptionTier::Lifetime
        )
    }

    #[allow(dead_code)]
    pub fn age(&self) -> Option<u16> {
        self.birth_year.map(|year| {
            let current_year = time::OffsetDateTime::now_utc().year();
            (current_year - year) as u16
        })
    }

    #[allow(dead_code)]
    pub fn can_show_profile_to_public(&self) -> bool {
        matches!(self.profile_visibility, ProfileVisibility::Public)
    }
}

impl UserProfileForCreate {
    #[allow(dead_code)]
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            first_name: None,
            last_name: None,
            bio: None,
            avatar_url: None,
            birth_year: None,
            gender: None,
            occupation: None,
            education_level: None,
            experience_level: None,
            timezone: None,
            country_code: None,
            language_preference: Some("en".to_string()),
            profile_visibility: Some(ProfileVisibility::Public),
            show_activity: Some(true),
            show_email: Some(false),
            subscription_tier: Some(SubscriptionTier::Free),
            registration_source: Some(RegistrationSource::Organic),
        }
    }

    #[allow(dead_code)]
    pub fn with_oauth_registration(user_id: Uuid, source: RegistrationSource) -> Self {
        let mut profile = Self::new(user_id);
        profile.registration_source = Some(source);
        profile
    }
}