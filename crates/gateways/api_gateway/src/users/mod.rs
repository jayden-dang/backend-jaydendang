use axum::{
    routing::{get, post},
    Router,
};
mod user_routes;
use jd_core::AppState;
use user_routes::create_user_route;

pub fn user_router() -> Router<AppState> {
    Router::new().route("/", post(create_user_route))
}
