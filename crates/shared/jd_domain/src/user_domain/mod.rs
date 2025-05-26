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
// <<<-- Region:: END    <<<---  User Gender

// -->>> Region:: START  --->>>  Education Level
#[derive(Debug, Clone, PartialEq, Serialize, Deen, Deserialize, Default)]
pub enum EducationLevel {
    HighSchool,
    Bachelor,
    Master,
    Doctorate,
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


impl PartialEq for AccountStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AccountStatus::Active, AccountStatus::Active) => true,
            (AccountStatus::Inactive, AccountStatus::Inactive) => true,
            (AccountStatus::Suspended, AccountStatus::Suspended) => true,
            (AccountStatus::PendingVerification, AccountStatus::PendingVerification) => true,
            (AccountStatus::Locked { until: u1, reason: r1 }, AccountStatus::Locked { until: u2, reason: r2 }) => {
                u1 == u2 && r1 == r2
            }
            (AccountStatus::MarkedForDeletion { scheduled_for: s1 }, AccountStatus::MarkedForDeletion { scheduled_for: s2 }) => {
                s1 == s2
            }
            _ => false,
        }
    }
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
