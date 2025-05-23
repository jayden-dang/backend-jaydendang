use axum::{middleware, routing::get, Router};
use dotenv::dotenv;
use tracing::{info, Level};

use api_gateway::mw::mw_res_map;
use jd_tracing::tracing_init;
use jd_utils::config;

mod error;

#[tokio::main]
async fn main() -> error::Result<()> {
    dotenv().ok();
    
    // Initialize tracing with debug level
    let _ = tracing_init();
    tracing::info!("Tracing initialized");

    let cfg = config::Config::from_env().expect("Loading env failed");

    info!("Loading Environment Success...");
    let app = Router::new()
        .route("/", get(root))
        .layer(middleware::map_response(mw_res_map::mw_map_response));
    info!("Server is running...");

    let listener = tokio::net::TcpListener::bind(cfg.web.addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
