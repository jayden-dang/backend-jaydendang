use axum::Router;
use jd_core::AppState;

mod error;
mod log;
pub mod middleware;
mod users;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn v1_routes(app_state: AppState) -> Router {
    Router::new().nest("/api/v1", Router::new()).with_state(app_state)
}
