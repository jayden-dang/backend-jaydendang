use jd_deencode::Deen;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub mod profile;
pub mod user;

// -->>> Region:: START  --->>>  User Gender
#[derive(Debug, Clone, PartialEq, Serialize, Deen, Deserialize)]
pub enum UserGender {
    Male,
    Female,
    Other,
    PreferNotToSay,
}
// <<<-- Region:: END    <<<---  User Gende

// -->>> Region:: START  --->>>  Education Level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Deen)]
pub enum EducationLevel {
    HighSchool,
    Bachelor,
    Master,
    PhD,
    Bootcamp,
    SelfTaught,
    #[default]
    Other,
}

// <<<-- Region:: END    <<<---  Education Level

// -->>> Region:: START  --->>>  Experience Level
#[derive(Debug, Clone, PartialEq, Serialize, Deen, Deserialize, Default)]
pub enum ExperienceLevel {
    #[default]
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}
// <<<-- Region:: END    <<<---  Experience Level

// -->>> Region:: START  --->>>  Subscription Tier
#[derive(Debug, Clone, PartialEq, Serialize, Deen, Deserialize, Default)]
pub enum SubscriptionTier {
    #[default]
    Free,
    Basic,
    Premium,
    Enterprise,
}

// <<<-- Region:: END    <<<---  Subscription Tier

// -->>> Region:: START  --->>>  Account Status
#[derive(Debug, Clone, Serialize, Deen, Deserialize, Default)]
pub enum AccountStatus {
    #[default]
    Active,
    Inactive,
    Suspended,
    PendingVerification,
    Locked {
        until: OffsetDateTime,
        reason: String,
    },
    MarkedForDeletion {
        scheduled_for: OffsetDateTime,
    },
}
// <<<-- Region:: END    <<<---  Account Status

// -->>> Region:: START  --->>>  Profile Visibility
#[derive(Debug, Clone, PartialEq, Serialize, Deen, Deserialize, Default)]
pub enum ProfileVisibility {
    #[default]
    Public,
    Private,
    Friends,
}
// <<<-- Region:: END    <<<---  Profile Visibility
