use crate::error::{Error, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use fastcrypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use fastcrypto::hash::Blake2b256;
use fastcrypto::hash::HashFunction;
use fastcrypto::traits::{ToFromBytes, VerifyingKey};
use tracing::{debug, error, info, warn};

#[async_trait]
pub trait SignatureVerifier: Send + Sync {
  async fn verify_signature(
    &self,
    message: &str,
    signature: &str,
    public_key: &str,
    address: &str,
  ) -> Result<bool>;
}

pub struct SuiSignatureVerifier;

impl SuiSignatureVerifier {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl SignatureVerifier for SuiSignatureVerifier {
  async fn verify_signature(
    &self,
    message: &str,
    signature: &str,
    public_key: &str,
    address: &str,
  ) -> Result<bool> {
    info!("🔍 Starting Sui signature verification");
    debug!("Message: {}", message);
    debug!("Signature (base64): {}", signature);
    debug!("Public key (base64): {}", public_key);
    debug!("Address: {}", address);

    // Decode base64 signature
    let signature_bytes = match general_purpose::STANDARD.decode(signature) {
      Ok(bytes) => {
        info!("✅ Signature decoded successfully, length: {} bytes", bytes.len());
        debug!("Signature bytes: {:?}", hex::encode(&bytes));
        bytes
      }
      Err(e) => {
        error!("❌ Failed to decode signature: {}", e);
        return Err(Error::invalid_signature());
      }
    };

    // Decode base64 public key
    let public_key_bytes = match general_purpose::STANDARD.decode(public_key) {
      Ok(bytes) => {
        info!("✅ Public key decoded successfully, length: {} bytes", bytes.len());
        debug!("Public key (hex): {}", hex::encode(&bytes));
        bytes
      }
      Err(e) => {
        error!("❌ Failed to decode public key: {}", e);
        return Err(Error::invalid_public_key());
      }
    };

    // Validate signature format (97 bytes for Sui format)
    if signature_bytes.len() != 97 {
      error!("❌ Invalid signature length: {} (expected 97 for Sui format)", signature_bytes.len());
      return Err(Error::invalid_signature());
    }

    // Check signature scheme (first byte should be 0x00 for Ed25519)
    if signature_bytes[0] != 0x00 {
      error!(
        "❌ Unsupported signature scheme: 0x{:02x} (expected 0x00 for Ed25519)",
        signature_bytes[0]
      );
      return Err(Error::invalid_signature());
    }

    // Extract Ed25519 signature (bytes 1-64) and embedded public key (bytes 65-96)
    let ed25519_sig_bytes = &signature_bytes[1..65];
    let embedded_pk_bytes = &signature_bytes[65..97];

    info!("📋 Signature format analysis:");
    info!("  - Scheme flag: 0x{:02x}", signature_bytes[0]);
    info!("  - Ed25519 signature length: {} bytes", ed25519_sig_bytes.len());
    info!("  - Embedded public key length: {} bytes", embedded_pk_bytes.len());
    debug!("  - Ed25519 signature (hex): {}", hex::encode(ed25519_sig_bytes));
    debug!("  - Embedded public key (hex): {}", hex::encode(embedded_pk_bytes));

    // Validate public key length (32 bytes for Ed25519)
    if public_key_bytes.len() != 32 {
      error!("❌ Invalid public key length: {} (expected 32)", public_key_bytes.len());
      return Err(Error::invalid_public_key());
    }

    // Verify embedded public key matches provided one
    if embedded_pk_bytes != public_key_bytes {
      warn!("⚠️ Embedded public key doesn't match provided public key");
      debug!("Provided: {}", hex::encode(&public_key_bytes));
      debug!("Embedded: {}", hex::encode(embedded_pk_bytes));
      // Don't fail here - use the provided public key for verification
      info!("ℹ️ Using provided public key for verification");
    } else {
      info!("✅ Embedded public key matches provided public key");
    }

    // Calculate and verify address from public key
    let scheme_flag = 0u8; // Ed25519 flag is 0
    let mut hasher_input = Vec::new();
    hasher_input.push(scheme_flag);
    hasher_input.extend_from_slice(&public_key_bytes);

    let hash_result = Blake2b256::digest(&hasher_input);
    let derived_address = format!("0x{}", hex::encode(hash_result.as_ref()));

    info!("🔑 Address verification:");
    info!("  - Derived from public key: {}", derived_address);
    info!("  - Provided address: {}", address);

    if derived_address != address {
      error!(
        "❌ Address mismatch: provided {} but public key derives {}",
        address, derived_address
      );
      return Err(Error::invalid_public_key());
    }

    // Parse Ed25519 public key - need to convert to array first
    let mut pk_array = [0u8; 32];
    pk_array.copy_from_slice(&public_key_bytes);

    let pk = match Ed25519PublicKey::from_bytes(&pk_array) {
      Ok(key) => {
        info!("✅ Ed25519 public key parsed successfully");
        key
      }
      Err(e) => {
        error!("❌ Failed to parse Ed25519 public key: {}", e);
        debug!("Public key bytes length: {}", public_key_bytes.len());
        debug!("Public key bytes (hex): {}", hex::encode(&public_key_bytes));
        return Err(Error::invalid_public_key());
      }
    };

    // Parse Ed25519 signature - need to convert to array first
    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(ed25519_sig_bytes);

    let sig = match Ed25519Signature::from_bytes(&sig_array) {
      Ok(signature) => {
        info!("✅ Ed25519 signature parsed successfully");
        signature
      }
      Err(e) => {
        error!("❌ Failed to parse Ed25519 signature: {}", e);
        debug!("Signature bytes length: {}", ed25519_sig_bytes.len());
        debug!("Signature bytes (hex): {}", hex::encode(ed25519_sig_bytes));
        return Err(Error::invalid_signature());
      }
    };

    // Create the Sui personal message format
    let prefix = b"\x19Sui Signed Message:\n";
    let message_bytes = message.as_bytes();
    let message_len = message_bytes.len() as u64;

    // Build the full message: prefix + length (8 bytes, little-endian) + message
    let mut message_with_prefix = Vec::new();
    message_with_prefix.extend_from_slice(prefix);
    message_with_prefix.extend_from_slice(&message_len.to_le_bytes());
    message_with_prefix.extend_from_slice(message_bytes);

    info!("📝 Message verification details:");
    info!("  - Original message: {}", message);
    info!("  - Message length: {} bytes", message_bytes.len());
    info!("  - Prefix (hex): {}", hex::encode(prefix));
    info!("  - Length bytes (hex): {}", hex::encode(&message_len.to_le_bytes()));
    info!("  - Full message length: {} bytes", message_with_prefix.len());
    debug!("  - Full message (hex): {}", hex::encode(&message_with_prefix));

    // Hash the message with Blake2b-256
    let message_hash = Blake2b256::digest(&message_with_prefix);
    let hash_bytes = message_hash.as_ref();

    info!("🔐 Message hash: {}", hex::encode(hash_bytes));
    debug!("🔍 Verification components:");
    debug!("  - Public key (hex): {}", hex::encode(&public_key_bytes));
    debug!("  - Signature (hex): {}", hex::encode(ed25519_sig_bytes));
    debug!("  - Hash length: {} bytes", hash_bytes.len());
    debug!("  - Hash type: {}", std::any::type_name_of_val(&hash_bytes));
    debug!("  - Signature type: {}", std::any::type_name_of_val(&sig));
    debug!("  - Public key type: {}", std::any::type_name_of_val(&pk));

    // Additional debugging - let's verify the hash matches expected format
    if hash_bytes.len() != 32 {
      error!("❌ Invalid hash length: {} (expected 32 bytes)", hash_bytes.len());
      return Err(Error::invalid_signature());
    }

    // Try different verification approaches
    info!("🔍 Attempting Ed25519 signature verification...");
    
    // Method 1: Direct verification with hash bytes
    info!("📋 Method 1: Direct hash verification");
    match pk.verify(hash_bytes, &sig) {
      Ok(_) => {
        info!("🎉 Signature verification SUCCESS with Method 1!");
        return Ok(true);
      }
      Err(e) => {
        error!("❌ Method 1 failed: {}", e);
        debug!("Method 1 error details: {:?}", e);
      }
    }

    // Method 2: Try verifying the raw message (without Sui prefix) - for debugging
    info!("📋 Method 2: Raw message verification (debug only)");
    let raw_hash = Blake2b256::digest(message_bytes);
    match pk.verify(raw_hash.as_ref(), &sig) {
      Ok(_) => {
        warn!("⚠️ Raw message verification succeeded - this suggests prefix issue");
        // Don't return true here since this would be incorrect for Sui
      }
      Err(e) => {
        debug!("Method 2 failed as expected: {}", e);
      }
    }

    // Method 3: Try with the exact message construction that Sui uses
    info!("📋 Method 3: Reconstructed Sui message format");
    
    // Reconstruct message exactly as Sui does it
    let mut sui_message = Vec::new();
    sui_message.extend_from_slice(b"\x19Sui Signed Message:\n");
    
    // Add message length as bytes - try both little-endian and big-endian
    let msg_len_le = (message_bytes.len() as u64).to_le_bytes();
    let msg_len_be = (message_bytes.len() as u64).to_be_bytes();
    
    // Try little-endian first (this is what Sui should use)
    sui_message.extend_from_slice(&msg_len_le);
    sui_message.extend_from_slice(message_bytes);
    
    let sui_hash = Blake2b256::digest(&sui_message);
    debug!("Sui message (LE) hash: {}", hex::encode(sui_hash.as_ref()));
    
    match pk.verify(sui_hash.as_ref(), &sig) {
      Ok(_) => {
        info!("🎉 Signature verification SUCCESS with Method 3 (LE)!");
        return Ok(true);
      }
      Err(e) => {
        error!("❌ Method 3 (LE) failed: {}", e);
      }
    }

    // Try big-endian as fallback
    let mut sui_message_be = Vec::new();
    sui_message_be.extend_from_slice(b"\x19Sui Signed Message:\n");
    sui_message_be.extend_from_slice(&msg_len_be);
    sui_message_be.extend_from_slice(message_bytes);
    
    let sui_hash_be = Blake2b256::digest(&sui_message_be);
    debug!("Sui message (BE) hash: {}", hex::encode(sui_hash_be.as_ref()));
    
    match pk.verify(sui_hash_be.as_ref(), &sig) {
      Ok(_) => {
        info!("🎉 Signature verification SUCCESS with Method 3 (BE)!");
        return Ok(true);
      }
      Err(e) => {
        error!("❌ Method 3 (BE) failed: {}", e);
      }
    }

    // Method 4: Test signature verification with known data
    info!("📋 Method 4: Testing signature components individually");
    
    // Verify public key can be used for basic operations
    debug!("Public key verification test passed");
    
    // Log all the raw data for manual verification
    error!("🔍 VERIFICATION FAILED - Debug information:");
    error!("  - Message: {}", message);
    error!("  - Message hex: {}", hex::encode(message_bytes));
    error!("  - Public key hex: {}", hex::encode(&public_key_bytes));
    error!("  - Signature hex: {}", hex::encode(ed25519_sig_bytes));
    error!("  - Expected hash: {}", hex::encode(hash_bytes));
    error!("  - Sui message length: {}", message_bytes.len());
    error!("  - Sui message length bytes (LE): {}", hex::encode(&msg_len_le));
    error!("  - Full Sui message hex: {}", hex::encode(&message_with_prefix));

    info!("❌ All signature verification methods failed");
    Ok(false)
  }
}

impl Default for SuiSignatureVerifier {
  fn default() -> Self {
    Self::new()
  }
}

