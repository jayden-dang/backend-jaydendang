use axum::{
    routing::{get, post},
    Router,
};
mod user_routes;
use jd_core::AppState;
use user_routes::{create_user_route, get_user_by_email, get_user_by_username, get_user_by_wow};

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_user_route).get(get_user_by_wow))
        .route("/{id}", get(get_user_by_username))
        .route("/email/{email}", get(get_user_by_email))
}
