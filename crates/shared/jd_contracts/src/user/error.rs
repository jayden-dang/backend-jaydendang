use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserServiceError {
  EmailAlreadyExists,
  UsernameTaken,
  InvalidPassword,
  NotFound,
}
