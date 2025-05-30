use axum::{
  extract::{Extension, Json},
  http::HeaderMap,
  response::Json as ResponseJson,
};
use std::sync::Arc;
use validator::Validate;

use crate::application::AuthService;
use crate::error::{Error, Result};
use crate::models::{
  NonceRequest, NonceResponse, RefreshRequest, RefreshResponse, UserInfo, VerifyRequest,
  VerifyResponse,
};

/// Generate nonce for wallet authentication
pub async fn generate_nonce(
  Extension(auth_service): Extension<Arc<AuthService>>,
  Json(request): Json<NonceRequest>,
) -> Result<ResponseJson<NonceResponse>> {
  // Validate request
  request
    .validate()
    .map_err(|e| Error::invalid_request_data(&format!("Validation failed: {}", e)))?;

  // Generate nonce
  let nonce = auth_service.generate_nonce(&request.address).await?;

  let response = NonceResponse { nonce: nonce.nonce.clone(), message: nonce.get_signing_message() };

  Ok(ResponseJson(response))
}

/// Verify wallet signature and authenticate user
pub async fn verify_signature(
  Extension(auth_service): Extension<Arc<AuthService>>,
  Json(request): Json<VerifyRequest>,
) -> Result<ResponseJson<VerifyResponse>> {
  // Validate request
  request
    .validate()
    .map_err(|e| Error::invalid_request_data(&format!("Validation failed: {}", e)))?;

  // Verify signature and authenticate
  let (user, tokens) = auth_service
    .verify_signature(&request.address, &request.signature, &request.public_key)
    .await?;

  let response = VerifyResponse { success: true, user: UserInfo::from(user), tokens };

  Ok(ResponseJson(response))
}


/// Refresh access token using refresh token
pub async fn refresh_token(
  Extension(auth_service): Extension<Arc<AuthService>>,
  Json(request): Json<RefreshRequest>,
) -> Result<ResponseJson<RefreshResponse>> {
  // Validate request
  request
    .validate()
    .map_err(|e| Error::invalid_request_data(&format!("Validation failed: {}", e)))?;

  // Refresh token
  let access_token = auth_service.refresh_token(&request.refresh_token).await?;

  let response = RefreshResponse { access_token };

  Ok(ResponseJson(response))
}

/// Middleware to validate JWT access token
pub async fn auth_middleware(
  Extension(auth_service): Extension<Arc<AuthService>>,
  headers: HeaderMap,
  mut request: axum::extract::Request,
) -> Result<axum::extract::Request> {
  // Get Authorization header
  let auth_header = headers
    .get("authorization")
    .and_then(|h| h.to_str().ok())
    .ok_or_else(|| Error::missing_auth_header())?;

  // Extract token
  let token = auth_service.extract_token_from_header(auth_header)?;

  // Validate token and get user
  let user = auth_service.validate_access_token(token).await?;

  // Add user to request extensions
  request.extensions_mut().insert(user);

  Ok(request)
}

/// Get current authenticated user info
pub async fn get_current_user(
  Extension(user): Extension<crate::domain::AuthUser>,
) -> ResponseJson<UserInfo> {
  ResponseJson(UserInfo::from(user))
}
