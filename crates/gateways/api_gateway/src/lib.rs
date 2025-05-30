use auth::auth_router;
use axum::Router;
use jd_core::AppState;
use users::user_router;

mod auth;
mod error;
mod log;
pub mod middleware;
mod routes_rpc;
mod sui;
mod users;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn v1_routes(app_state: AppState) -> Router {
  let mm = app_state.mm.as_ref().clone();
  Router::new()
    .nest(
      "/api/v1",
      Router::new()
        .nest("/users", user_router())
        .nest("/sui", sui::sui_router())
        .nest("/auth", auth_router()),
    )
    .nest("/api", routes_rpc::routes(mm))
    .with_state(app_state)
}
