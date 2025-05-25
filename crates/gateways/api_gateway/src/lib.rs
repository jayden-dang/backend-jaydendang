use axum::Router;
use jd_core::ModelManager;
use users::{profile_routes, user_routes};

mod error;
mod log;
pub mod middleware;
mod users;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn v1_routes(mm: ModelManager) -> Router {
    Router::new()
        .nest("/api/v1", Router::new().merge(user_routes()).merge(profile_routes()))
        .with_state(mm)
}
