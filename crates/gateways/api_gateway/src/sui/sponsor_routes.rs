use axum::{
  Router,
  routing::get,
};
use jd_core::AppState;

use super::Handler;

// Sponsor operations - simplified to working methods only
pub fn sponsor_router() -> Router<AppState> {
  Router::new()
    .route("/health", get(Handler::health_check))
    .route("/test-connection", get(Handler::test_connection))
}
