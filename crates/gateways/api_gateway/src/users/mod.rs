use axum::{
    routing::{get, post},
    Router,
};
use jd_core::AppState;
use user_service::users::{
    application::handlers::UserHandler,
    infrastructure::UserRepositoryImpl,
};

type Handler = UserHandler<UserRepositoryImpl>;

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/", post(Handler::create_user).get(Handler::get_user_by_wow))
        .route("/{id}", get(Handler::get_user_by_username))
        .route("/email/{email}", get(Handler::get_user_by_email))
}

