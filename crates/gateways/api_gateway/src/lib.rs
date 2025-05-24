use axum::Router;
use routes::route_login::login_routes;

mod error;
mod log;
pub mod middleware;
pub mod routes;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn v1_routes() -> Router {
    Router::new().nest("/api/v1", Router::new().merge(login_routes()))
}
