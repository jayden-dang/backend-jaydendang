use api_gateway::{
    middleware::{mw_auth::mw_ctx_resolve, mw_res_map, mw_res_timestamp},
    v1_routes,
};

use axum::{http::StatusCode, middleware, response::IntoResponse, Json, Router};
use dotenv::dotenv;
use jd_core::ModelManager;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tracing::info;

use jd_tracing::tracing_init;
use jd_utils::{
    config,
    time::{format_time, now_utc},
};

mod error;

#[tokio::main]
async fn main() -> error::Result<()> {
    dotenv().ok();

    let _ = tracing_init();

    let mm = ModelManager::new().await.expect("");

    let cfg = config::Config::from_env().expect("Loading env failed");

    // TODO: -- Convert to AppState Model
    let app = Router::new()
        .merge(v1_routes(mm.clone()))
        .layer(middleware::map_response(mw_res_map::mw_map_response))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .layer(middleware::from_fn(mw_res_timestamp::mw_req_stamp_resolver))
        .fallback(fallback_handler);
    info!("Server is running on port: {}", cfg.web.addr);

    let listener = tokio::net::TcpListener::bind(cfg.web.addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
