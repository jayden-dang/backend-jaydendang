// use jd_core::ctx::Ctx;
// use crate::web::{self, ClientError};
// use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Result, mw::mw_res_timestamp::ReqStamp};
use axum::http::{Method, Uri};
use jd_utils::time::{format_time, now_utc};
use serde::Serialize;
use serde_json::{Value, json};
use serde_with::skip_serializing_none;
use tracing::{debug, error};

// TODO: Add CTX & web Client Errro
pub async fn log_request(
    uri: Uri,
    req_method: Method,
    req_stamp: ReqStamp,
    log_data: Value,
    status: u8,
    // ctx: Option<Ctx>,
    // web_error: Option<&Error>,
    // client_error: Option<ClientError>,
) -> Result<()> {
    // TODO: Add Ctx and Web Error
    // let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

    // let error_type = web_error.map(|se| se.as_ref().to_string());
    // let error_data = serde_json::to_value(web_error)
    //     .ok()
    //     .and_then(|mut v| v.get_mut("data").map(|v| v.take()));
    let ReqStamp { uuid, time_in } = req_stamp;
    let now = now_utc();

    let log = RequestLogLine {
        // Basic request identification
        uuid: uuid.to_string(),
        timestamp: format_time(now),
        time_in: format_time(time_in),
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
    uuid: String,      // uuid string formatted
    timestamp: String, // Rfc 3339
    time_in: String,   // Rfc 3339
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
