use api_gateway::{
    middleware::{mw_auth::mw_ctx_resolve, mw_res_map, mw_res_timestamp},
    routes::route_login::routes,
};

use axum::{http::StatusCode, middleware, response::IntoResponse, routing::get, Json, Router};
use dotenv::dotenv;
use jd_core::ModelManager;
use serde_json::{json, Value};
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

    let app = Router::new()
        .route("/", get(root))
        .merge(routes())
        .layer(middleware::map_response(mw_res_map::mw_map_response))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .layer(middleware::from_fn(mw_res_timestamp::mw_req_stamp_resolver))
        .fallback(fallback_handler);
    info!("Server is running... on port: {}", cfg.web.addr);

    let listener = tokio::net::TcpListener::bind(cfg.web.addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

// basic handler that responds with a static string
async fn root() -> Json<Value> {
    Json(json!({
        "data": "Hello, World!"
    }))
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
