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
mod routes_rpc;

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
