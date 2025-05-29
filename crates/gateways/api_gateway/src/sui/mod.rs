use axum::{
  routing::get,
  Router,
};
mod sponsor_routes;
use jd_core::AppState;
use sui_service::application::handlers::sui_handler::SuiHandler;
use sui_service::infrastructure::sui_repository_impl::SuiRepositoryImpl;

type Handler = SuiHandler<SuiRepositoryImpl>;

pub fn sui_router() -> Router<AppState> {
  Router::new()
    // Coin operations
    .route("/", get(Handler::fetch_coin))
    .merge(sponsor_routes::sponsor_router())
}

