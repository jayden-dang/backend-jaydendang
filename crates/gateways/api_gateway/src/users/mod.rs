use axum::{
    routing::{get, post},
    Router,
};
use jd_core::AppState;
use user_service::users::repository::create_user;

pub fn user_router() -> Router<AppState> {
    Router::new().route("/", post(create_user))
}
