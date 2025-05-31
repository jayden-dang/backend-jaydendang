use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use auth_service::domain::UserRole;
use super::user_profile::{UserProfile, UserProfileForCreate, UserProfileForUpdate};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UnifiedUser {
    // Core identity from unified_auth.users
    pub user_id: Uuid,
    pub email: Option<String>,
    pub username: String,
    pub display_name: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
    pub is_email_verified: bool,
    pub is_profile_complete: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_login: Option<OffsetDateTime>,
    pub login_count: i32,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    
    // Profile information (optional, loaded via JOIN)
    pub profile: Option<UserProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UnifiedUserForCreate {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(max = 150))]
    pub display_name: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
    pub is_email_verified: Option<bool>,
    
    // Optional profile data
    pub profile: Option<UserProfileForCreate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UnifiedUserForUpdate {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 3, max = 50))]
    pub username: Option<String>,
    #[validate(length(max = 150))]
    pub display_name: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
    pub is_email_verified: Option<bool>,
    pub is_profile_complete: Option<bool>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_login: Option<OffsetDateTime>,
    pub login_count: Option<i32>,
    
    // Profile updates
    pub profile: Option<UserProfileForUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
    pub is_premium: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_login: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUserProfile {
    pub user_id: Uuid,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub experience_level: Option<String>,
    pub occupation: Option<String>,
    pub location: Option<String>, // Derived from country_code
    pub is_premium: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub member_since: OffsetDateTime,
}

impl UnifiedUser {
    #[allow(dead_code)]
    pub fn is_soft_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    #[allow(dead_code)]
    pub fn can_access_role(&self, required_role: UserRole) -> bool {
        self.is_active && self.role.can_access(required_role)
    }

    #[allow(dead_code)]
    pub fn is_premium(&self) -> bool {
        self.profile
            .as_ref()
            .map(|p| p.is_premium())
            .unwrap_or(false) || self.role.is_premium()
    }

    #[allow(dead_code)]
    pub fn full_name(&self) -> Option<String> {
        self.profile.as_ref().map(|p| p.display_name())
    }

    #[allow(dead_code)]
    pub fn effective_display_name(&self) -> String {
        self.display_name
            .clone()
            .or_else(|| self.full_name())
            .unwrap_or_else(|| self.username.clone())
    }

    #[allow(dead_code)]
    pub fn is_profile_public(&self) -> bool {
        self.profile
            .as_ref()
            .map(|p| p.can_show_profile_to_public())
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn to_summary(&self) -> UserSummary {
        UserSummary {
            user_id: self.user_id,
            username: self.username.clone(),
            display_name: Some(self.effective_display_name()),
            avatar_url: self.profile.as_ref().and_then(|p| p.avatar_url.clone()),
            role: self.role,
            is_active: self.is_active,
            is_premium: self.is_premium(),
            created_at: self.created_at,
            last_login: self.last_login,
        }
    }

    #[allow(dead_code)]
    pub fn to_public_profile(&self) -> Option<PublicUserProfile> {
        if !self.is_profile_public() {
            return None;
        }

        let profile = self.profile.as_ref()?;
        
        let location = profile.country_code.as_ref().map(|code| {
            // Simple country code to name mapping - in production use a proper library
            match code.as_str() {
                "US" => "United States",
                "CA" => "Canada",
                "GB" => "United Kingdom",
                "DE" => "Germany",
                "FR" => "France",
                "JP" => "Japan",
                "AU" => "Australia",
                _ => "Unknown",
            }.to_string()
        });

        Some(PublicUserProfile {
            user_id: self.user_id,
            username: self.username.clone(),
            display_name: Some(self.effective_display_name()),
            bio: profile.bio.clone(),
            avatar_url: profile.avatar_url.clone(),
            experience_level: profile.experience_level.map(|l| format!("{:?}", l)),
            occupation: profile.occupation.clone(),
            location,
            is_premium: self.is_premium(),
            member_since: self.created_at,
        })
    }

    #[allow(dead_code)]
    pub fn validate_username(username: &str) -> bool {
        username.len() >= 3 && 
        username.len() <= 50 && 
        username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') &&
        !username.chars().all(|c| c.is_numeric()) // Prevent all-numeric usernames
    }

    #[allow(dead_code)]
    pub fn validate_display_name(display_name: &str) -> bool {
        !display_name.trim().is_empty() && display_name.len() <= 150
    }
}

impl UnifiedUserForCreate {
    #[allow(dead_code)]
    pub fn new(username: String, email: Option<String>) -> Self {
        Self {
            username,
            email,
            display_name: None,
            role: Some(UserRole::Normal),
            is_active: Some(true),
            is_email_verified: Some(false),
            profile: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_oauth_data(
        username: String,
        email: String,
        display_name: Option<String>,
        avatar_url: Option<String>,
    ) -> Self {
        let profile = UserProfileForCreate {
            user_id: Uuid::new_v4(), // Will be replaced with actual user_id
            first_name: None,
            last_name: None,
            bio: None,
            avatar_url,
            birth_year: None,
            gender: None,
            occupation: None,
            education_level: None,
            experience_level: None,
            timezone: None,
            country_code: None,
            language_preference: Some("en".to_string()),
            profile_visibility: None,
            show_activity: Some(true),
            show_email: Some(false),
            subscription_tier: None,
            registration_source: None,
        };

        Self {
            username,
            email: Some(email),
            display_name,
            role: Some(UserRole::Normal),
            is_active: Some(true),
            is_email_verified: Some(true), // OAuth emails are verified
            profile: Some(profile),
        }
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        if !UnifiedUser::validate_username(&self.username) {
            return Err("Invalid username format".to_string());
        }

        if let Some(email) = &self.email {
            if !email.contains('@') || email.len() > 255 {
                return Err("Invalid email format".to_string());
            }
        }

        if let Some(display_name) = &self.display_name {
            if !UnifiedUser::validate_display_name(display_name) {
                return Err("Invalid display name".to_string());
            }
        }

        Ok(())
    }
}