use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use hyper::StatusCode;
use jd_core::AppState;
use serde::Serialize;
use serde_json::{json, Value};
use user_service::users::repository::create_user;

mod error;
mod log;
pub mod middleware;
mod users;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn v1_routes(app_state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", Router::new().route("/users", post(create_user)))
        .with_state(app_state)
}
