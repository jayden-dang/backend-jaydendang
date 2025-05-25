use std::fmt::Display;

use jd_utils::regex::USERNAME_REGEX;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::Id;

use super::AccountStatus;

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: Id,
    pub email: Email,
    pub username: Username,
    pub password_hash: HashedPassword,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    // pub status: AccountStatus,
    pub email_verified: bool,
}

// -->>> Region:: START  --->>>  Email
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Validate)]
pub struct Email {
    #[validate(email)]
    pub value: String,
    pub domain: String,
    pub local_part: String,
}

impl From<Email> for sea_query::Value {
    fn from(email: Email) -> Self {
        sea_query::Value::String(Some(Box::new(email.value)))
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Email {
    pub fn new(email: String) -> Result<Self, ValidationError> {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(ValidationError::new("invalid_email"));
        }

        let local_part = parts[0].to_string();
        let domain = parts[1].to_string();

        let email = Email {
            value: email,
            domain,
            local_part,
        };

        email.validate().map_err(|_| ValidationError::new("invalid_email"))?;
        Ok(email)
    }
}

// <<<-- Region:: END    <<<---  Email

// -->>> Region:: START  --->>>  Username
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Validate)]
pub struct Username {
    #[validate(length(min = 3, max = 50), regex(path = "USERNAME_REGEX"))]
    pub value: String,
}

impl From<Username> for sea_query::Value {
    fn from(username: Username) -> Self {
        sea_query::Value::String(Some(Box::new(username.value)))
    }
}

impl Username {
    pub fn new(username: String) -> Result<Self, ValidationError> {
        let username = Username { value: username };
        username
            .validate()
            .map_err(|_| ValidationError::new("invalid_username"))?;
        Ok(username)
    }
}
// <<<-- Region:: END    <<<---  Username

// -->>> Region:: START  --->>>  Password
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HashedPassword {
    pub value: String,
}

impl From<HashedPassword> for sea_query::Value {
    fn from(value: HashedPassword) -> Self {
        sea_query::Value::String(Some(Box::new(value.value)))
    }
}

impl HashedPassword {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
// <<<-- Region:: END    <<<---  Password

// Domain validation trait
pub trait DomainValidation {
    type Error;
    fn validate_domain(&self) -> Result<(), Self::Error>;
}

impl DomainValidation for User {
    type Error = ValidationError;

    fn validate_domain(&self) -> Result<(), Self::Error> {
        self.email
            .validate()
            .map_err(|_| ValidationError::new("invalid_email"))?;
        self.username
            .validate()
            .map_err(|_| ValidationError::new("invalid_username"))?;

        // Business logic validation
        if let (Some(first_name), Some(last_name)) = (&self.first_name, &self.last_name) {
            if first_name.is_empty() || last_name.is_empty() {
                return Err(ValidationError::new("Name cannot be empty"));
            }
        }

        Ok(())
    }
}
