use axum::{
  extract::{Extension, Json, State},
  http::HeaderMap,
  response::Json as ResponseJson,
};
use validator::Validate;

use crate::application::use_cases::{
  GenerateNonceUseCase, RefreshTokenUseCase, ValidateTokenUseCase, VerifySignatureUseCase,
};
use crate::domain::{AuthUser, NonceRepository, SignatureVerifier, UserRepository};
use crate::error::{Error, Result};
use crate::infrastructure::{NonceRepositoryImpl, SignatureVerifierImpl, UserRepositoryImpl};
use crate::models::{
  NonceRequest, NonceResponse, RefreshRequest, RefreshResponse, UserInfo, VerifyRequest,
  VerifyResponse,
};
use jd_core::AppState;

pub struct AuthHandler<N: NonceRepository, U: UserRepository, S: SignatureVerifier> {
  pub generate_nonce: GenerateNonceUseCase<N>,
  pub verify_signature: VerifySignatureUseCase<N, U, S>,
  pub refresh_token: RefreshTokenUseCase,
  pub validate_token: ValidateTokenUseCase<U>,
}

impl<N: NonceRepository, U: UserRepository, S: SignatureVerifier> AuthHandler<N, U, S> {
  pub fn new(
    generate_nonce: GenerateNonceUseCase<N>,
    verify_signature: VerifySignatureUseCase<N, U, S>,
    refresh_token: RefreshTokenUseCase,
    validate_token: ValidateTokenUseCase<U>,
  ) -> Self {
    Self { generate_nonce, verify_signature, refresh_token, validate_token }
  }

  pub async fn generate_nonce(
    State(state): State<AppState>,
    Json(request): Json<NonceRequest>,
  ) -> Result<ResponseJson<NonceResponse>> {
    request
      .validate()
      .map_err(|e| Error::invalid_request_data(&format!("Validation failed: {}", e)))?;

    let nonce_repo = NonceRepositoryImpl::new(state);
    let use_case = GenerateNonceUseCase::new(nonce_repo);
    let nonce = use_case.execute(&request.address).await?;

    let response =
      NonceResponse { nonce: nonce.nonce.clone(), message: nonce.get_signing_message() };

    Ok(ResponseJson(response))
  }

  pub async fn verify_signature(
    State(state): State<AppState>,
    Json(request): Json<VerifyRequest>,
  ) -> Result<ResponseJson<VerifyResponse>> {
    request
      .validate()
      .map_err(|e| Error::invalid_request_data(&format!("Validation failed: {}", e)))?;

    let nonce_repo = NonceRepositoryImpl::new(state.clone());
    let user_repo = UserRepositoryImpl::new(state.clone());
    let signature_verifier = SignatureVerifierImpl::new();
    let jwt_secret = state.config.auth_jwt_secret.clone();

    let use_case =
      VerifySignatureUseCase::new(nonce_repo, user_repo, signature_verifier, jwt_secret);

    let (user, tokens) = use_case
      .execute(&request.address, &request.signature, &request.public_key)
      .await?;

    let response = VerifyResponse { success: true, user: UserInfo::from(user), tokens };

    Ok(ResponseJson(response))
  }

  pub async fn refresh_token(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
  ) -> Result<ResponseJson<RefreshResponse>> {
    request
      .validate()
      .map_err(|e| Error::invalid_request_data(&format!("Validation failed: {}", e)))?;

    let jwt_secret = state.config.auth_jwt_secret.clone();
    let use_case = RefreshTokenUseCase::new(jwt_secret);
    let access_token = use_case.execute(&request.refresh_token).await?;

    let response = RefreshResponse { access_token };

    Ok(ResponseJson(response))
  }

  pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: axum::extract::Request,
  ) -> Result<axum::extract::Request> {
    let auth_header = headers
      .get("authorization")
      .and_then(|h| h.to_str().ok())
      .ok_or_else(Error::missing_auth_header)?;

    let user_repo = UserRepositoryImpl::new(state.clone());
    let jwt_secret = state.config.auth_jwt_secret.clone();
    let use_case = ValidateTokenUseCase::new(user_repo, jwt_secret);

    let token = ValidateTokenUseCase::<UserRepositoryImpl>::extract_token_from_header(auth_header)?;
    let user = use_case.execute(token).await?;

    request.extensions_mut().insert(user);

    Ok(request)
  }

  pub async fn get_current_user(Extension(user): Extension<AuthUser>) -> ResponseJson<UserInfo> {
    ResponseJson(UserInfo::from(user))
  }
}
