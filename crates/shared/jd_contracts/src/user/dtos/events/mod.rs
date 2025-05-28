use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
  pub user_id: Uuid,
  pub email: String,
  pub created_at: OffsetDateTime,
}
