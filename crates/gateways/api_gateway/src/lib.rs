use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use jd_contracts::user::dto::UserFilter;
use jd_core::{base::rest, ModelManager};
use routes::route_login::api_login_handler;
use std::sync::Arc;
use user_service::infra::{record::UserRecord, UserDmc};
use uuid::Uuid;

mod error;
mod log;
pub mod middleware;
pub mod routes;

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn v1_routes(mm: ModelManager) -> Router {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/login", post(api_login_handler))
                .route("/users", get(get_user_by_id)),
        )
        .with_state(mm)
}

pub async fn get_user_by_id(
    Query(query): Query<UserFilter>,
    State(mm): State<ModelManager>,
) -> Result<Json<UserRecord>> {
    if query.email.is_none() && query.username.is_none() {
        return Err(error::Error::BadRequest(
            "At least one filter criteria must be provided".to_string(),
        ));
    }
    Ok(Json(
        rest::get_by_sth::<UserDmc, _, _>(&mm, Some(query))
            .await
            .map_err(|e| error::Error::CoreError(Arc::new(e)))?,
    ))
}
