use axum::{
    routing::{get, post},
    Router,
};
use jd_core::AppState;
use user_service::users::application::handlers::create_user;

pub fn user_router() -> Router<AppState> {
    Router::new().route("/", post(create_user))
}
