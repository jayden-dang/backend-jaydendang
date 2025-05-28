use crate::error::{ClientError, Error};
use crate::{Result, middleware::mw_res_timestamp::ReqStamp};
use axum::http::{Method, Uri};
use jd_core::ctx::Ctx;
use jd_utils::time::{format_time, now_utc};
use serde::Serialize;
use serde_json::{Value, json};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use time::Duration;
use tracing::info;

// List of sensitive fields that should be masked
const SENSITIVE_FIELDS: &[&str] = &[
  "password",
  "pwd",
  "token",
  "secret",
  "key",
  "credit_card",
  "card_number",
  "cvv",
  "ssn",
  "social_security",
  "phone",
  "email",
];

/// Request information for logging
#[derive(Debug)]
pub struct LogRequest {
  pub uri: Uri,
  pub method: Method,
  pub stamp: ReqStamp,
  pub ctx: Option<Ctx>,
  pub body: Option<Value>,
}

/// Response information for logging
#[derive(Debug)]
pub struct LogResponse {
  pub body: Option<Value>,
  pub error: Option<Error>,
  pub client_error: Option<ClientError>,
}

/// Complete log entry containing request and response information
#[derive(Debug)]
pub struct LogEntry {
  pub request: LogRequest,
  pub response: LogResponse,
}

impl LogEntry {
  pub fn new(request: LogRequest, response: LogResponse) -> Self {
    Self { request, response }
  }
}

fn sanitize_value(value: &mut Value) {
  match value {
    Value::Object(map) => {
      for (key, val) in map.iter_mut() {
        if SENSITIVE_FIELDS
          .iter()
          .any(|&field| key.to_lowercase().contains(field))
        {
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

fn extract_query_params(uri: &Uri) -> Option<Value> {
  uri.query().map(|q| {
    let params: HashMap<String, String> = q
      .split('&')
      .filter_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next()?.to_string();
        let value = parts.next().unwrap_or("").to_string();
        Some((key, value))
      })
      .collect();

    let mut value = json!(params);
    sanitize_value(&mut value);
    value
  })
}

pub async fn log_request(log_entry: LogEntry) -> Result<()> {
  let LogEntry { request, response } = log_entry;

  let error_type = response.error.as_ref().map(|e| e.as_ref().to_string());
  let error_data = response
    .error
    .as_ref()
    .and_then(|e| serde_json::to_value(e).ok())
    .and_then(|mut v| v.get_mut("data").map(|v| v.take()));

  let ReqStamp { uuid, time_in } = request.stamp;
  let now = now_utc();
  let duration: Duration = now - time_in;
  let duration_ms = (duration.as_seconds_f64() * 1_000_000.).floor() / 1_000.;

  // Calculate response size before using response_body
  let response_size = response.body.as_ref().map(|b| b.to_string().len());

  // Extract and sanitize query parameters
  let query_params = extract_query_params(&request.uri);

  // Sanitize request body if present
  let mut sanitized_request_body = request.body;
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
      method: request.method.to_string(),
      path: request.uri.path().to_string(),
      query: query_params,
      headers: None,
      body: sanitized_request_body,
      user_id: request.ctx.map(|c| c.user_id()),
    },

    // Response context
    response: ResponseContext {
      status: if response.error.is_some() { "error" } else { "success" }.to_string(),
      body: response.body,
      size: response_size,
    },

    // Error context (if any)
    error: if response.error.is_some() {
      Some(ErrorContext {
        type_: error_type,
        client_type: response.client_error.map(|e| e.message),
        data: error_data,
      })
    } else {
      None
    },
  };

  info!("REQUEST LOG: \n {}", json!(log));
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

// Usage examples:
/*
// Method 1: Using LogEntry struct
let request = LogRequest {
    uri,
    method: req_method,
    stamp: req_stamp,
    ctx,
    body: request_body,
};

let response = LogResponse {
    body: response_body,
    error: web_error,
    client_error,
};

let log_entry = LogEntry::new(request, response);
log_request(log_entry).await?;

// Method 2: Using Builder pattern
LogRequestBuilder::new()
    .uri(uri)
    .method(req_method)
    .stamp(req_stamp)
    .ctx(ctx)
    .web_error(web_error)
    .client_error(client_error)
    .request_body(request_body)
    .response_body(response_body)
    .log()
    .await?;
*/
