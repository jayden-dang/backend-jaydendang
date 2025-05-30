use crate::Result;
use crate::error::Error;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::Request;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use jd_core::{AppState, ctx::Ctx};
use serde::Serialize;
use tower_cookies::Cookies;

#[allow(dead_code)] // For now, until we have the rpc.
pub async fn mw_ctx_require(ctx: Result<CtxW>, req: Request<Body>, next: Next) -> Result<Response> {
  ctx?;

  Ok(next.run(req).await)
}

#[allow(unused_variables, unused_mut)] // For now, until we have the rpc.
pub async fn mw_ctx_resolve(
  State(app_state): State<AppState>,
  cookies: Cookies,
  mut req: Request<Body>,
  next: Next,
) -> Result<Response> {
  let ctx_ext_result = ctx_resolve(app_state, &cookies).await;

  // Add context to request extensions
  if let Ok(ctx_w) = ctx_ext_result {
    req.extensions_mut().insert(ctx_w.0);
  }

  Ok(next.run(req).await)
}

async fn ctx_resolve(_app_state: AppState, _cookies: &Cookies) -> CtxExtResult {
  Ctx::new(0i64)
    .map(CtxW)
    .map_err(|_| CtxExtError::CtxCreateFail("error".to_string()))
}

// region:    --- Ctx Extractor
#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

impl<S: Send + Sync> FromRequestParts<S> for CtxW {
  type Rejection = Error;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
    parts
      .extensions
      .get::<CtxExtResult>()
      .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
      .clone()
      .map_err(Error::CtxExt)
  }
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
  TokenNotInCookie,
  TokenWrongFormat,

  UserNotFound,
  ModelAccessError(String),
  FailValidate,
  CannotSetTokenCookie,

  CtxNotInRequestExt,
  CtxCreateFail(String),
}

impl std::fmt::Display for CtxExtError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::TokenNotInCookie => write!(f, "Token not found in cookie"),
      Self::TokenWrongFormat => write!(f, "Token has wrong format"),
      Self::UserNotFound => write!(f, "User not found"),
      Self::ModelAccessError(msg) => write!(f, "Model access error: {}", msg),
      Self::FailValidate => write!(f, "Validation failed"),
      Self::CannotSetTokenCookie => write!(f, "Cannot set token cookie"),
      Self::CtxNotInRequestExt => write!(f, "Context not found in request extensions"),
      Self::CtxCreateFail(msg) => write!(f, "Failed to create context: {}", msg),
    }
  }
}

impl std::error::Error for CtxExtError {}
// endregion: --- Ctx Extractor Result/Error
