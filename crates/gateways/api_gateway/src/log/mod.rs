// use jd_core::ctx::Ctx;
// use crate::web::{self, ClientError};
// use std::time::{SystemTime, UNIX_EPOCH};

use crate::{mw::mw_res_timestamp::ReqStamp, Result};
use axum::http::{Method, Uri};
use jd_utils::time::{format_time, now_utc};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use time::Duration;
use tracing::debug;

// TODO: Add CTX & web Client Errro
pub async fn log_request(
    uri: Uri,
    req_method: Method,
    req_stamp: ReqStamp,
    log_data: Option<Value>,
    // ctx: Option<Ctx>,
    // web_error: Option<&Error>,
    // client_error: Option<ClientError>,
) -> Result<()> {
    // TODO: Add Ctx and Web Error

    // let error_type = web_error.map(|se| se.as_ref().to_string());
    // let error_data = serde_json::to_value(web_error)
    //     .ok()
    //     .and_then(|mut v| v.get_mut("data").map(|v| v.take()));
    let ReqStamp { uuid, time_in } = req_stamp;
    let now = now_utc();
    let duration: Duration = now - time_in;
    let duration_ms = (duration.as_seconds_f64() * 1_000_000.).floor() / 1_000.;

    let log = RequestLogLine {
        // Basic request identification
        uuid: uuid.to_string(),
        timestamp: format_time(now),
        time_in: format_time(time_in),
        request_id: uuid.to_string(),

        duration_ms,

        response_data: log_data,

        // Request details
        http_path: uri.path().to_string(),
        http_method: req_method.to_string(),
        query_params: uri.query().map(|q| q.to_string()),
        request_headers: None,
        // -- error attributes
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

    // Duration
    duration_ms: f64,

    // Request details
    http_path: String,
    http_method: String,
    query_params: Option<String>,
    request_headers: Option<Value>,

    response_data: Option<Value>,
    // TODO: More Information
    // client_ip: String,
    // headers: String
}
