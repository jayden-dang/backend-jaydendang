use axum::{
  routing::{get, post},
  Router,
};
use jd_core::AppState;
use sui_service::application::handlers::sui_handler::SuiHandler;
use sui_service::infrastructure::sui_repository_impl::SuiRepositoryImpl;

type Handler = SuiHandler<SuiRepositoryImpl>;

pub fn sui_router() -> Router<AppState> {
  Router::new().route("/", post(Handler::fetch_coin))
}

