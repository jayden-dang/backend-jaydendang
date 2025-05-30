use api_gateway::{
  middleware::{
    mw_auth::mw_ctx_resolve, mw_request_context::mw_request_context, mw_res_map, mw_res_timestamp,
  },
  v1_routes,
};

use axum::{http::StatusCode, middleware, response::IntoResponse, Json, Router};
use dotenv::dotenv;
use jd_core::AppState;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing::info;

use jd_tracing::tracing_init;
use jd_utils::{
  config,
  time::{format_time, now_utc},
};

use axum::http::{HeaderName, HeaderValue, Method};

mod error;

#[tokio::main]
async fn main() -> error::Result<()> {
  dotenv().ok();

  let _ = tracing_init();

  let app_state = AppState::new().await.expect("Failed to create app state");

  let cfg = config::Config::from_env().expect("Loading env failed");

  let app = Router::new()
    .merge(v1_routes(app_state.clone()))
    .layer(middleware::map_response(mw_res_map::mw_map_response))
    .layer(middleware::from_fn_with_state(app_state.clone(), mw_ctx_resolve))
    .layer(CookieManagerLayer::new())
    .layer(middleware::from_fn(mw_res_timestamp::mw_req_stamp_resolver))
    .layer(middleware::from_fn(mw_request_context))
    .layer(
      CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([
          HeaderName::from_static("content-type"),
          HeaderName::from_static("authorization"),
          HeaderName::from_static("accept"),
          HeaderName::from_static("x-requested-with"),
          HeaderName::from_static("x-request-id"),
          HeaderName::from_static("x-trace-id"),
        ])
        .allow_credentials(true),
    )
    .fallback(fallback_handler);

  info!("Server is running on port: {}", cfg.web.addr);

  let listener = tokio::net::TcpListener::bind(cfg.web.addr).await.unwrap();
  axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
    .await
    .unwrap();
  Ok(())
}

// Professional fallback handler for unmatched routes
async fn fallback_handler() -> impl IntoResponse {
  let response = json!({
      "status": "error",
      "code": 404,
      "message": "Route not found",
      "timestamp": format_time(now_utc()),
      "path": "The requested resource does not exist"
  });

  (StatusCode::NOT_FOUND, Json(response))
}
