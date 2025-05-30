use auth_service::{
  application::handlers::AuthHandler,
  domain::AuthUser,
  infrastructure::{NonceRepositoryImpl, SignatureVerifierImpl, UserRepositoryImpl},
  models::{
    NonceRequest, NonceResponse, RefreshRequest, RefreshResponse, UserInfo, VerifyRequest,
    VerifyResponse,
  },
};
use axum::{
  Router,
  extract::{Extension, Json, State},
  response::Json as ResponseJson,
  routing::{get, post},
};
use jd_core::AppState;

pub fn auth_router() -> Router<AppState> {
  Router::new()
    .route("/nonce", post(generate_nonce))
    .route("/verify", post(verify_signature))
    .route("/refresh", post(refresh_token))
    .route("/me", get(get_current_user))
}

async fn generate_nonce(
  State(state): State<AppState>,
  Json(request): Json<NonceRequest>,
) -> auth_service::Result<ResponseJson<NonceResponse>> {
  AuthHandler::<NonceRepositoryImpl, UserRepositoryImpl, SignatureVerifierImpl>::generate_nonce(
    State(state),
    Json(request),
  )
  .await
}

async fn verify_signature(
  State(state): State<AppState>,
  Json(request): Json<VerifyRequest>,
) -> auth_service::Result<ResponseJson<VerifyResponse>> {
  AuthHandler::<NonceRepositoryImpl, UserRepositoryImpl, SignatureVerifierImpl>::verify_signature(
    State(state),
    Json(request),
  )
  .await
}

async fn refresh_token(
  State(state): State<AppState>,
  Json(request): Json<RefreshRequest>,
) -> auth_service::Result<ResponseJson<RefreshResponse>> {
  AuthHandler::<NonceRepositoryImpl, UserRepositoryImpl, SignatureVerifierImpl>::refresh_token(
    State(state),
    Json(request),
  )
  .await
}

async fn get_current_user(Extension(user): Extension<AuthUser>) -> ResponseJson<UserInfo> {
  AuthHandler::<NonceRepositoryImpl, UserRepositoryImpl, SignatureVerifierImpl>::get_current_user(
    Extension(user),
  )
  .await
}
