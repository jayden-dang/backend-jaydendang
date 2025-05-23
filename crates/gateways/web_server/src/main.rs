use axum::{middleware, routing::get, Router, Json, extract::Query};
use dotenv::dotenv;
use tracing::{info, Level};
use serde_json::json;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use api_gateway::mw::{mw_res_map, pagination::PaginationMetadata};
use serde::Deserialize;

use jd_tracing::tracing_init;
use jd_utils::config;

mod error;

#[derive(Debug, Deserialize)]
struct PaginationParams {
    page: Option<u32>,
    per_page: Option<u32>,
    order_by: Option<String>,
    order_direction: Option<String>,
}

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
        .route("/api/posts", get(list_posts))
        .route("/api/posts/paginated", get(list_posts_paginated))
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

// Sample handler for testing pagination
async fn list_posts() -> impl IntoResponse {
    // Sample data
    let posts = vec![
        json!({"id": 1, "title": "Post 1", "content": "Content 1"}),
        json!({"id": 2, "title": "Post 2", "content": "Content 2"}),
        json!({"id": 3, "title": "Post 3", "content": "Content 3"}),
        json!({"id": 4, "title": "Post 4", "content": "Content 4"}),
        json!({"id": 5, "title": "Post 5", "content": "Content 5"}),
    ];

    // Create pagination metadata
    let pagination = PaginationMetadata::new_offset(
        1,    // current_page
        2,    // per_page
        3,    // total_pages
        5,    // total_items
        true, // has_next
        false // has_prev
    ).with_order("id".to_string(), "desc".to_string());

    // Create response
    let mut response = (StatusCode::OK, Json(json!({
        "data": posts,
        "meta": {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "content_type": "application/json",
            "pagination": pagination
        }
    }))).into_response();

    // Add pagination to response extensions
    response.extensions_mut().insert(pagination);

    response
}

// Enhanced handler with more test data and pagination
async fn list_posts_paginated(params: Query<PaginationParams>) -> impl IntoResponse {
    // Generate more sample data
    let all_posts: Vec<serde_json::Value> = (1..=50).map(|i| {
        json!({
            "id": i,
            "title": format!("Post {}", i),
            "content": format!("Content {}", i),
            "author": format!("Author {}", (i % 5) + 1),
            "category": format!("Category {}", (i % 3) + 1),
            "created_at": chrono::Utc::now().to_rfc3339(),
            "views": i * 100,
            "likes": i * 10
        })
    }).collect();

    // Get pagination parameters with defaults
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    let order_by = params.order_by.clone().unwrap_or_else(|| "id".to_string());
    let order_direction = params.order_direction.clone().unwrap_or_else(|| "desc".to_string());

    // Calculate pagination values
    let total_items = all_posts.len() as u64;
    let total_pages = (total_items as f64 / per_page as f64).ceil() as u32;
    let has_next = page < total_pages;
    let has_prev = page > 1;

    // Sort posts based on order_by and order_direction
    let mut sorted_posts = all_posts.clone();
    sorted_posts.sort_by(|a, b| {
        let a_val = a.get(&order_by);
        let b_val = b.get(&order_by);
        
        // Handle numeric values
        if let (Some(a_num), Some(b_num)) = (a_val.and_then(|v| v.as_u64()), b_val.and_then(|v| v.as_u64())) {
            if order_direction == "desc" {
                b_num.cmp(&a_num)
            } else {
                a_num.cmp(&b_num)
            }
        }
        // Handle string values
        else if let (Some(a_str), Some(b_str)) = (a_val.and_then(|v| v.as_str()), b_val.and_then(|v| v.as_str())) {
            if order_direction == "desc" {
                b_str.cmp(a_str)
            } else {
                a_str.cmp(b_str)
            }
        }
        // Default comparison
        else {
            std::cmp::Ordering::Equal
        }
    });

    // Get paginated slice
    let start = ((page - 1) * per_page) as usize;
    let end = std::cmp::min(start + per_page as usize, sorted_posts.len());
    let paginated_posts = &sorted_posts[start..end];

    // Create pagination metadata
    let pagination = PaginationMetadata::new_offset(
        page,
        per_page,
        total_pages,
        total_items,
        has_next,
        has_prev
    ).with_order(order_by, order_direction);

    // Create response
    let mut response = (StatusCode::OK, Json(json!({
        "data": paginated_posts,
        "meta": {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "content_type": "application/json",
            "pagination": pagination
        }
    }))).into_response();

    // Add pagination to response extensions
    response.extensions_mut().insert(pagination);

    response
}
