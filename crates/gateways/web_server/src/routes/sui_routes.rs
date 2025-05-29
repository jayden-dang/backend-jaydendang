use axum::{Json, extract::State, response::IntoResponse};
use jd_core::AppState;
use serde_json::json;

pub async fn sui_routes() -> axum::Router {
  axum::Router::new()
    .route("/api/v1/sui/version", axum::routing::get(get_sui_version))
    .with_state(AppState::new().await.unwrap())
}

async fn get_sui_version(State(state): State<AppState>) -> impl IntoResponse {
  let sui_client = state.sui_client();
  match sui_client.get_api_version().await {
    Ok(version) => Json(json!({
        "status": "success",
        "data": {
            "version": version
        }
    }))
    .into_response(),
    Err(e) => Json(json!({
        "status": "error",
        "message": e.to_string()
    }))
    .into_response(),
  }
}
