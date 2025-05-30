use axum::{Router, routing::post};
mod sponsor_routes;
use jd_core::AppState;
use sui_service::application::handlers::sui_handler::SuiHandler;
use sui_service::infrastructure::enhanced_sui_repository::EnhancedSuiRepository;

type Handler = SuiHandler<EnhancedSuiRepository>;

pub fn sui_router() -> Router<AppState> {
  Router::new()
    // Coin operations  
    .route("/fetch-coin", post(Handler::fetch_coin))
    .merge(sponsor_routes::sponsor_router())
}
