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

// List of sensitive fields that should be masked
const SENSITIVE_FIELDS: &[&str] = &[
    "password", "pwd", "token", "secret", "key",
    "credit_card", "card_number", "cvv",
    "ssn", "social_security",
    "phone", "email",
];

fn sanitize_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if SENSITIVE_FIELDS.iter().any(|&field| key.to_lowercase().contains(field)) {
                    *val = json!("[REDACTED]");
                } else {
                    sanitize_value(val);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                sanitize_value(item);
            }
        }
        _ => {}
    }
}

pub async fn log_request(
    uri: Uri,
    req_method: Method,
    req_stamp: ReqStamp,
    ctx: Option<Ctx>,
    web_error: Option<&Error>,
    client_error: Option<ClientError>,
    request_body: Option<Value>,
    response_body: Option<Value>,
) -> Result<()> {
    let error_type = web_error.map(|se| se.as_ref().to_string());
    let error_data = serde_json::to_value(web_error)
        .ok()
        .and_then(|mut v| v.get_mut("data").map(|v| v.take()));

    let ReqStamp { uuid, time_in } = req_stamp;
    let now = now_utc();
    let duration: Duration = now - time_in;
    let duration_ms = (duration.as_seconds_f64() * 1_000_000.).floor() / 1_000.;

    // Calculate response size before using response_body
    let response_size = response_body.as_ref().map(|b| b.to_string().len());

    // Extract and sanitize query parameters
    let query_params: Option<Value> = uri.query().map(|q| {
        let mut params: std::collections::HashMap<_, _> = q
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next().unwrap_or("");
                Some((key.to_string(), value.to_string()))
            })
            .collect();
        let mut value = json!(params);
        sanitize_value(&mut value);
        value
    });

    // Sanitize request body if present
    let mut sanitized_request_body = request_body;
    if let Some(ref mut body) = sanitized_request_body {
        sanitize_value(body);
    }

    let log = RequestLogLine {
        // Request identification
        id: uuid.to_string(),
        timestamp: format_time(now),
        duration_ms,

        // Request context
        request: RequestContext {
            method: req_method.to_string(),
            path: uri.path().to_string(),
            query: query_params,
            headers: None,
            body: sanitized_request_body,
            user_id: ctx.map(|c| c.user_id()),
        },

        // Response context
        response: ResponseContext {
            status: if web_error.is_some() { "error" } else { "success" }.to_string(),
            body: response_body,
            size: response_size,
        },

        // Error context (if any)
        error: if web_error.is_some() {
            Some(ErrorContext {
                type_: error_type,
                client_type: client_error.map(|e| e.as_ref().to_string()),
                data: error_data,
            })
        } else {
            None
        },
    };

    debug!("REQUEST LOG: \n {}", json!(log));
    Ok(())
}

#[skip_serializing_none]
#[derive(Serialize)]
struct RequestLogLine {
    // Request identification
    id: String,
    timestamp: String,
    duration_ms: f64,

    // Request context
    request: RequestContext,

    // Response context
    response: ResponseContext,

    // Error context (if any)
    error: Option<ErrorContext>,
}

#[skip_serializing_none]
#[derive(Serialize)]
struct RequestContext {
    method: String,
    path: String,
    query: Option<Value>,
    headers: Option<Value>,
    body: Option<Value>,
    user_id: Option<i64>,
}

#[skip_serializing_none]
#[derive(Serialize)]
struct ResponseContext {
    status: String,
    body: Option<Value>,
    size: Option<usize>,
}

#[skip_serializing_none]
#[derive(Serialize)]
struct ErrorContext {
    #[serde(rename = "type")]
    type_: Option<String>,
    client_type: Option<String>,
    data: Option<Value>,
}
