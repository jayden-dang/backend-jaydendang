use axum::Router;
use jd_core::AppState;

pub fn sui_router() -> Router<AppState> {
  Router::new()
  // Read operations
  // .route("/objects/:object_id", get(get_object))
}
