use axum::{
  routing::{get, post},
  Router,
};
use jd_core::AppState;
use sui_service::application::handlers::sui_handler::SuiHandler;
use sui_service::infrastructure::sui_repository_impl::SuiRepositoryImpl;

type Handler = SuiHandler<SuiRepositoryImpl>;

pub fn sui_router() -> Router<AppState> {
  Router::new()
    // Coin operations
    .route("/", get(Handler::fetch_coin))
    
    // Gas Station operations
    .route("/health", get(Handler::health_check))
    .route("/debug", get(Handler::debug_config))
    .route("/test-gas-station", get(Handler::test_gas_station))
    .route("/sponsor", post(Handler::sponsor_transaction))
    .route("/gas-pool-status", get(Handler::gas_pool_status))
    .route("/user/{address}/stats", get(Handler::user_stats))
    .route("/refresh-gas-pool", post(Handler::refresh_gas_pool))
}

