// use jd_core::ctx::Ctx;
// use crate::web::{self, ClientError};
// use std::time::{SystemTime, UNIX_EPOCH};

use crate::Result;
use axum::http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use tracing::{debug, error};
use uuid::Uuid;

// TODO: Add CTX & web Client Errro
pub async fn log_request(uuid: Uuid, uri: Uri, req_method: Method, log_data: Value, status: u8) -> Result<()> {
    // let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let now = chrono::Utc::now();

    let log = RequestLogLine {
        // Basic request identification
        log_type: "request".to_string(),
        timestamp: now.to_rfc3339(),
        request_id: uuid.to_string(),

        // Request details
        http_path: uri.path().to_string(),
        http_method: req_method.to_string(),
        query_params: uri.query().map(|q| q.to_string()),
        request_headers: None,

        // Response details
        status_code: status,
        response_time_ms: log_data["response"]["time_ms"].as_u64().unwrap_or(0),
        response_size_bytes: log_data["response"]["size_bytes"].as_u64().unwrap_or(0),
        response_data: Some(log_data.clone()),

        // Error tracking
        error_type: if status == 0 { Some("error".to_string()) } else { None },
        error_details: if status == 0 { Some(log_data) } else { None },
        stack_trace: None,

        // Environment info
        environment: std::env::var("RUST_ENV").ok(),
        service_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    if status == 0 {
        error!("Error Request Log: {}", json!(log));
    } else {
        debug!("Request Log: {}", json!(log));
    }
    Ok(())
}

#[skip_serializing_none]
#[derive(Serialize)]
struct RequestLogLine {
    // Basic request identification
    log_type: String,
    timestamp: String,
    request_id: String,

    // Request details
    http_path: String,
    http_method: String,
    query_params: Option<String>,
    request_headers: Option<Value>,

    // Response details
    status_code: u8,
    response_time_ms: u64,
    response_size_bytes: u64,
    response_data: Option<Value>,

    // Error tracking
    error_type: Option<String>,
    error_details: Option<Value>,
    stack_trace: Option<String>,

    // Environment info
    environment: Option<String>,
    service_version: String,
}
