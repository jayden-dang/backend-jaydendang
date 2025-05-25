use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
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
                .route("/user/{id}", get(get_user_by_id)),
        )
        .with_state(mm)
}

pub async fn get_user_by_id(State(mm): State<ModelManager>, Path(id): Path<String>) -> Result<Json<UserRecord>> {
    let uuid = Uuid::parse_str(&id).map_err(|_| error::Error::EntityNotFound { entity: "user", id: 0 })?;
    Ok(Json(
        rest::get_by_id::<UserDmc, _>(&mm, uuid)
            .await
            .map_err(|e| error::Error::CoreError(Arc::new(e)))?,
    ))
}
