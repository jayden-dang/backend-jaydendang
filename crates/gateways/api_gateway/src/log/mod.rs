use crate::error::{ClientError, Error};
use crate::{middleware::mw_res_timestamp::ReqStamp, Result};
use axum::http::{Method, Uri};
use jd_core::ctx::Ctx;
use jd_utils::time::{format_time, now_utc};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use time::Duration;
use tracing::debug;

// TODO: web Client Error
pub async fn log_request(
    uri: Uri,
    req_method: Method,
    req_stamp: ReqStamp,
    // log_data: Option<Value>,
    ctx: Option<Ctx>,
    web_error: Option<&Error>,
    client_error: Option<ClientError>,
) -> Result<()> {
    // TODO: Add Ctx and Web Error

    let error_type = web_error.map(|se| se.as_ref().to_string());
    let error_data = serde_json::to_value(web_error)
        .ok()
        .and_then(|mut v| v.get_mut("data").map(|v| v.take()));

    let ReqStamp { uuid, time_in } = req_stamp;
    let now = now_utc();
    let duration: Duration = now - time_in;
    let duration_ms = (duration.as_seconds_f64() * 1_000_000.).floor() / 1_000.;

    // Extract query parameters
    let query_params: Option<Value> = uri.query().map(|q| {
        let params: std::collections::HashMap<_, _> = q
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next().unwrap_or("");
                Some((key.to_string(), value.to_string()))
            })
            .collect();
        json!(params)
    });

    let log = RequestLogLine {
        // Basic request identification
        uuid: uuid.to_string(),
        timestamp: format_time(now),
        time_in: format_time(time_in),
        request_id: uuid.to_string(),

        // Performance metrics
        duration_ms,
        user_id: ctx.map(|c| c.user_id()),

        // Request details
        http_path: uri.path().to_string(),
        http_method: req_method.to_string(),
        query_params,
        request_headers: None,

        // Error information
        client_error_type: client_error.map(|e| e.as_ref().to_string()),
        error_data,
        error_type,

        // Additional context
        status: if web_error.is_some() { "error" } else { "success" }.to_string(),
        user_agent: None,    // TODO: Extract from headers
        ip_address: None,    // TODO: Extract from request
        response_size: None, // TODO: Calculate from response
    };

    debug!("REQUEST LOG: \n {}", json!(log));
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

    // Performance metrics
    duration_ms: f64,
    user_id: Option<i64>,

    // Request details
    http_path: String,
    http_method: String,
    query_params: Option<Value>,
    request_headers: Option<Value>,

    // response_data: Option<Value>,
    // TODO: More Information
    // client_ip: String,
    // headers: String
    // -- Errors attributes.
    client_error_type: Option<String>,
    error_type: Option<String>,
    error_data: Option<Value>,

    // Additional context
    status: String,
    user_agent: Option<String>,
    ip_address: Option<String>,
    response_size: Option<usize>,
}
