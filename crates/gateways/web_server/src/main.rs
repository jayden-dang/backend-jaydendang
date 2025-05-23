use api_gateway::mw::mw_res_map;
use axum::{middleware, routing::get, Router, response::IntoResponse, http::StatusCode, Json};
use dotenv::dotenv;
use tracing::info;
use serde_json::json;

use jd_tracing::tracing_init;
use jd_utils::config;

mod error;

#[tokio::main]
async fn main() -> error::Result<()> {
    dotenv().ok();

    let _ = tracing_init();
    tracing::info!("Tracing initialized");

    let cfg = config::Config::from_env().expect("Loading env failed");

    info!("Loading Environment Success...");
    let app = Router::new()
        .route("/", get(root))
        .layer(middleware::map_response(mw_res_map::mw_map_response))
        .fallback(fallback_handler);
    info!("Server is running...");

    let listener = tokio::net::TcpListener::bind(cfg.web.addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

// Professional fallback handler for unmatched routes
async fn fallback_handler() -> impl IntoResponse {
    let response = json!({
        "status": "error",
        "code": 404,
        "message": "Route not found",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "path": "The requested resource does not exist"
    });

    (StatusCode::NOT_FOUND, Json(response))
}
