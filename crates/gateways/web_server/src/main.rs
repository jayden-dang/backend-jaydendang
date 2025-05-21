use axum::{routing::get, Router};
use dotenv::dotenv;
use tracing::info;

use jd_core::config;
use jd_tracing::tracing_init;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cfg = config::Config::from_env().expect("Loading env failed");

    let _ = tracing_init();

    let app = Router::new().route("/", get(root));
    info!("Server is running...");

    let listener = tokio::net::TcpListener::bind(cfg.web.addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
