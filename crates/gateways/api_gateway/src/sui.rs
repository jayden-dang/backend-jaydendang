use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use jd_core::AppState;
use serde_json::json;

pub fn sui_router() -> Router<AppState> {
  Router::new().route("/version", get(get_sui_version))
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
