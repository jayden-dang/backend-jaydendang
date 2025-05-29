use axum::{
  routing::{get, post},
  Router,
};
use jd_core::AppState;

use super::Handler;

// Sponsor operations
pub fn sponsor_router() -> Router<AppState> {
  Router::new()
    .route("/health", get(Handler::health_check))
    .route("/debug", get(Handler::debug_config))
    .route("/gas-station", get(Handler::gas_station))
    .route("/sponsor", post(Handler::sponsor_transaction))
    .route("/gas-pool-status", get(Handler::gas_pool_status))
    .route("/user/{address}/stats", get(Handler::user_stats))
    .route("/refresh-gas-pool", post(Handler::refresh_gas_pool))
}
