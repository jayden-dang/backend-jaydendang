use axum::{
  Router,
  routing::{post, get},
  extract::{Json, Extension},
  response::Json as ResponseJson,
  http::{StatusCode, HeaderMap},
};
use jd_core::AppState;
use auth_service::{
  application::AuthService,
  models::{NonceRequest, VerifyRequest, RefreshRequest, NonceResponse, VerifyResponse, RefreshResponse},
};
use std::sync::Arc;
use serde_json::json;

pub fn auth_router() -> Router<AppState> {
  Router::new()
    .route("/nonce", post(handle_generate_nonce))
    .route("/verify", post(handle_verify_signature))
    .route("/refresh", post(handle_refresh_token))
    .route("/me", get(handle_get_current_user))
}

// Nonce generation handler
async fn handle_generate_nonce(
  Extension(auth_service): Extension<Arc<AuthService>>,
  Json(payload): Json<NonceRequest>,
) -> Result<ResponseJson<NonceResponse>, StatusCode> {
  match auth_service.generate_nonce(&payload.address).await {
    Ok(nonce) => Ok(ResponseJson(NonceResponse {
      nonce: nonce.nonce.clone(),
      message: nonce.get_signing_message(),
    })),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}

// Signature verification handler  
async fn handle_verify_signature(
  Extension(auth_service): Extension<Arc<AuthService>>,
  Json(payload): Json<VerifyRequest>,
) -> Result<ResponseJson<VerifyResponse>, StatusCode> {
  match auth_service.verify_signature(&payload.address, &payload.signature, &payload.public_key).await {
    Ok((user, tokens)) => Ok(ResponseJson(VerifyResponse {
      success: true,
      user: user.into(),
      tokens,
    })),
    Err(_) => Err(StatusCode::UNAUTHORIZED),
  }
}

// Token refresh handler
async fn handle_refresh_token(
  Extension(auth_service): Extension<Arc<AuthService>>,
  Json(payload): Json<RefreshRequest>, 
) -> Result<ResponseJson<RefreshResponse>, StatusCode> {
  match auth_service.refresh_token(&payload.refresh_token).await {
    Ok(access_token) => Ok(ResponseJson(RefreshResponse {
      access_token,
    })),
    Err(_) => Err(StatusCode::UNAUTHORIZED),
  }
}

// Get current user handler (with JWT authentication)
async fn handle_get_current_user(
  Extension(auth_service): Extension<Arc<AuthService>>,
  headers: HeaderMap,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
  // Get Authorization header
  let auth_header = headers
    .get("authorization")
    .and_then(|h| h.to_str().ok())
    .ok_or(StatusCode::UNAUTHORIZED)?;

  // Extract and validate token
  let token = match auth_service.extract_token_from_header(auth_header) {
    Ok(token) => token,
    Err(_) => return Err(StatusCode::UNAUTHORIZED),
  };

  // Get user from token
  let user = match auth_service.validate_access_token(token).await {
    Ok(user) => user,
    Err(_) => return Err(StatusCode::UNAUTHORIZED),
  };

  Ok(ResponseJson(json!({
    "user": {
      "address": user.address,
      "public_key": user.public_key,
      "created_at": user.created_at,
      "last_login": user.last_login,
      "login_count": user.login_count,
    }
  })))
}
