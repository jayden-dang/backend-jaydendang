use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nonce {
  pub address: String,
  pub nonce: String,
  pub created_at: DateTime<Utc>,
  pub expires_at: DateTime<Utc>,
}

impl Nonce {
  /// Generate a new nonce for the given address
  pub fn generate(address: String) -> Self {
    let nonce = Self::generate_nonce_string();
    let now = Utc::now();
    let expires_at = now + Duration::minutes(5); // 5 minute expiration

    Self { address, nonce, created_at: now, expires_at }
  }

  /// Check if the nonce has expired
  pub fn is_expired(&self) -> bool {
    Utc::now() > self.expires_at
  }

  /// Generate the message that should be signed
  pub fn get_signing_message(&self) -> String {
    format!("Sign this message to authenticate with Commandos HKT: {}", self.nonce)
  }

  /// Generate a cryptographically secure 64-character hex string (32 bytes)
  fn generate_nonce_string() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.r#gen();
    bytes
      .iter()
      .map(|b| format!("{:02x}", b))
      .collect::<String>()
  }

  /// Validate nonce format (64 hex characters)
  pub fn is_valid_format(nonce: &str) -> bool {
    nonce.len() == 64 && nonce.chars().all(|c| c.is_ascii_hexdigit())
  }
}
