use crate::users::user_rpc;
use axum::Router;
use axum::response::IntoResponse;
use axum::{Json, extract::State, routing::post};
use jd_core::ctx::Ctx;
use jd_core::{AppState, ModelManager};
use serde_json::{Value, json};

/// Simple RPC handler that routes to the appropriate function
pub async fn simple_rpc_handler(
  State(app_state): State<AppState>,
  Json(rpc_req): Json<Value>,
) -> impl IntoResponse {
  // Create a default context for now
  let ctx = Ctx::new(0).unwrap_or_else(|_| Ctx::root_ctx());
  // Extract method and params from the request
  let method = rpc_req.get("method").and_then(|v| v.as_str()).unwrap_or("");

  let params = rpc_req.get("params").cloned().unwrap_or(json!({}));

  let id = rpc_req.get("id").cloned();

  // Route to the appropriate handler
  let result = match method {
    m if m.starts_with("user.") => {
      let user_method = &m[5..]; // Remove "user." prefix
      user_rpc::handle_user_rpc(user_method, params, ctx, app_state.clone()).await
    }
    _ => Err(jd_core::Error::RpcError(format!("Unknown method: {}", method))),
  };

  // Build the response
  match result {
    Ok(data) => Json(json!({
        "jsonrpc": "2.0",
        "result": data,
        "id": id
    })),
    Err(e) => Json(json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32603,
            "message": e.to_string()
        },
        "id": id
    })),
  }
}

/// Build the Axum router for '/api/rpc'
pub fn routes(_mm: ModelManager) -> Router<AppState> {
  Router::new().route("/rpc", post(simple_rpc_handler))
}
