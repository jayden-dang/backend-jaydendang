use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Normal,
    Member,
    Vip,
    Moderator,
    Admin,
}

impl UserRole {
    pub fn all() -> Vec<UserRole> {
        vec![
            UserRole::Normal,
            UserRole::Member,
            UserRole::Vip,
            UserRole::Moderator,
            UserRole::Admin,
        ]
    }

    pub fn level(&self) -> u8 {
        match self {
            UserRole::Normal => 0,
            UserRole::Member => 1,
            UserRole::Vip => 2,
            UserRole::Moderator => 3,
            UserRole::Admin => 4,
        }
    }

    pub fn can_access(&self, required_role: UserRole) -> bool {
        self.level() >= required_role.level()
    }

    pub fn is_staff(&self) -> bool {
        matches!(self, UserRole::Moderator | UserRole::Admin)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn is_premium(&self) -> bool {
        matches!(self, UserRole::Vip | UserRole::Moderator | UserRole::Admin)
    }
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let role_str = match self {
            UserRole::Normal => "normal",
            UserRole::Member => "member",
            UserRole::Vip => "vip",
            UserRole::Moderator => "moderator",
            UserRole::Admin => "admin",
        };
        write!(f, "{}", role_str)
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Normal
    }
}

impl From<UserRole> for sea_query::Value {
    fn from(role: UserRole) -> Self {
        sea_query::Value::String(Some(Box::new(role.to_string())))
    }
}

impl sea_query::Nullable for UserRole {
    fn null() -> sea_query::Value {
        sea_query::Value::String(None)
    }
}