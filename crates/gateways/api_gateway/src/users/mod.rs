use axum::{
    routing::{get, post},
    Router,
};
use jd_contracts::user::api::{CREATE_USER_PATH, GET_USER_PATH};
use jd_core::ModelManager;
use user_service::users::repository::create_user;

pub fn user_routes() -> Router<ModelManager> {
    Router::new().route(CREATE_USER_PATH, post(create_user))
}

pub fn profile_routes() -> Router<ModelManager> {
    Router::new().route(GET_USER_PATH, get(create_user))
}
