pub mod user_rpc;

use axum::{
  Router,
  routing::{get, post},
};
use jd_core::AppState;
use user_service::{
  application::handlers::user_handler::UserHandler,
  infrastructure::database::user_repository_impl::UserRepositoryImpl,
};

type Handler = UserHandler<UserRepositoryImpl>;

pub fn user_router() -> Router<AppState> {
  Router::new()
    .route("/", post(Handler::create_user).get(Handler::get_user_by_wow))
    .route("/{id}", get(Handler::get_user_by_username))
    .route("/email/{email}", get(Handler::get_user_by_email))
}
