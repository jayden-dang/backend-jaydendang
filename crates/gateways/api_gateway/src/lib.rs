use axum::Router;
use jd_core::AppState;
use users::user_router;
use auth::auth_router;

mod error;
mod log;
pub mod middleware;
mod sui;
mod users;
mod auth;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn v1_routes(app_state: AppState) -> Router {
  Router::new()
    .nest(
      "/api/v1",
      Router::new()
        .nest("/users", user_router())
        .nest("/sui", sui::sui_router())
        .nest("/auth", auth_router()),
    )
    .with_state(app_state)
}
