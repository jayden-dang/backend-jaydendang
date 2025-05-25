use sea_query::Value;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub mod profile;
pub mod user;

// -->>> Region:: START  --->>>  User Gender
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum UserGender {
    Male,
    Female,
    Other,
    PreferNotToSay,
}

impl From<UserGender> for Value {
    fn from(gender: UserGender) -> Self {
        Value::String(Some(Box::new(gender.to_string())))
    }
}

impl std::fmt::Display for UserGender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserGender::Male => write!(f, "male"),
            UserGender::Female => write!(f, "female"),
            UserGender::Other => write!(f, "other"),
            UserGender::PreferNotToSay => write!(f, "prefer_not_to_say"),
        }
    }
}
// <<<-- Region:: END    <<<---  User Gender

// -->>> Region:: START  --->>>  Education Level
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum EducationLevel {
    HighSchool,
    Bachelor,
    Master,
    Doctorate,
    Other,
}

impl From<EducationLevel> for Value {
    fn from(level: EducationLevel) -> Self {
        Value::String(Some(Box::new(level.to_string())))
    }
}

impl std::fmt::Display for EducationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EducationLevel::HighSchool => write!(f, "high_school"),
            EducationLevel::Bachelor => write!(f, "bachelor"),
            EducationLevel::Master => write!(f, "master"),
            EducationLevel::Doctorate => write!(f, "doctorate"),
            EducationLevel::Other => write!(f, "other"),
        }
    }
}
// <<<-- Region:: END    <<<---  Education Level

// -->>> Region:: START  --->>>  Experience Level
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

impl From<ExperienceLevel> for Value {
    fn from(level: ExperienceLevel) -> Self {
        Value::String(Some(Box::new(level.to_string())))
    }
}

impl std::fmt::Display for ExperienceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExperienceLevel::Beginner => write!(f, "beginner"),
            ExperienceLevel::Intermediate => write!(f, "intermediate"),
            ExperienceLevel::Advanced => write!(f, "advanced"),
            ExperienceLevel::Expert => write!(f, "expert"),
        }
    }
}
// <<<-- Region:: END    <<<---  Experience Level

// -->>> Region:: START  --->>>  Subscription Tier
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SubscriptionTier {
    Free,
    Basic,
    Premium,
    Enterprise,
}

impl From<SubscriptionTier> for Value {
    fn from(tier: SubscriptionTier) -> Self {
        Value::String(Some(Box::new(tier.to_string())))
    }
}

impl std::fmt::Display for SubscriptionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionTier::Free => write!(f, "free"),
            SubscriptionTier::Basic => write!(f, "basic"),
            SubscriptionTier::Premium => write!(f, "premium"),
            SubscriptionTier::Enterprise => write!(f, "enterprise"),
        }
    }
}
// <<<-- Region:: END    <<<---  Subscription Tier

// -->>> Region:: START  --->>>  Account Status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
    Locked { until: OffsetDateTime, reason: String },
    MarkedForDeletion { scheduled_for: OffsetDateTime },
}

impl From<AccountStatus> for Value {
    fn from(status: AccountStatus) -> Self {
        Value::String(Some(Box::new(status.to_string())))
    }
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStatus::Active => write!(f, "active"),
            AccountStatus::Inactive => write!(f, "inactive"),
            AccountStatus::Suspended => write!(f, "suspended"),
            AccountStatus::PendingVerification => write!(f, "pending_verification"),
            AccountStatus::Locked { until, reason } => write!(f, "locked:{}:{}", until, reason),
            AccountStatus::MarkedForDeletion { scheduled_for } => write!(f, "marked_for_deletion:{}", scheduled_for),
        }
    }
}
// <<<-- Region:: END    <<<---  Account Status

// -->>> Region:: START  --->>>  Profile Visibility
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ProfileVisibility {
    Public,
    Private,
    Friends,
}

impl From<ProfileVisibility> for Value {
    fn from(visibility: ProfileVisibility) -> Self {
        Value::String(Some(Box::new(visibility.to_string())))
    }
}

impl std::fmt::Display for ProfileVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileVisibility::Public => write!(f, "public"),
            ProfileVisibility::Private => write!(f, "private"),
            ProfileVisibility::Friends => write!(f, "friends"),
        }
    }
}
// <<<-- Region:: END    <<<---  Profile Visibility
