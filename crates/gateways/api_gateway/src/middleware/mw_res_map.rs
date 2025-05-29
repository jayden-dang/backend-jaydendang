use crate::Result;
use crate::{
  error::Error,
  log::{log_request, LogEntry, LogRequest, LogResponse},
};
use axum::body::to_bytes;
use axum::{
  http::{Method, StatusCode, Uri},
  response::{IntoResponse, Response},
  Json,
};
use jd_utils::time::{format_time, now_utc};
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;

use super::{mw_auth::CtxW, mw_res_timestamp::ReqStamp};
use crate::error::RequestContext;

/// Standard response structure for all API responses
#[derive(Debug)]
struct ApiResponse {
  id: String,
  status: u8, // 0 for success, 1 for error
  response_type: String,
  data: Option<Value>,
  error: Option<Value>,
  meta: Value,
}

impl ApiResponse {
  fn success(id: Uuid, data: Value) -> Self {
    Self {
      id: id.to_string(),
      status: 0,
      response_type: "success".to_string(),
      data: Some(data),
      error: None,
      meta: json!({
          "timestamp": format_time(now_utc())
      }),
    }
  }

  fn error(id: Uuid, error_data: Value) -> Self {
    Self {
      id: id.to_string(),
      status: 1,
      response_type: "error".to_string(),
      data: None,
      error: Some(error_data),
      meta: json!({
          "timestamp": format_time(now_utc())
      }),
    }
  }

  fn to_json(&self) -> Value {
    json!({
        "id": self.id,
        "status": self.status,
        "type": self.response_type,
        "data": self.data,
        "error": self.error,
        "meta": self.meta
    })
  }
}

/// Response processing result containing status and body
#[derive(Debug)]
struct ProcessedResponse {
  status_code: StatusCode,
  body: Value,
  client_error: Option<crate::error::ClientError>,
}

/// Extract and parse response body into a Value
async fn extract_response_body(body: axum::body::Body) -> Value {
  let bytes = to_bytes(body, usize::MAX).await.unwrap_or_default();
  let body_string = String::from_utf8(bytes.to_vec()).unwrap_or_default();
  serde_json::from_str(&body_string).unwrap_or(Value::Null)
}

/// Process successful response
fn process_success_response(uuid: Uuid, data: Value) -> ProcessedResponse {
  let api_response = ApiResponse::success(uuid, data);
  ProcessedResponse {
    status_code: StatusCode::OK,
    body: api_response.to_json(),
    client_error: None,
  }
}

/// Process error response from web_error
fn process_web_error_response(uuid: Uuid, web_error: &Error) -> ProcessedResponse {
  let request_context = RequestContext::default();
  let (status_code, client_error) = web_error.client_status_and_error(&request_context);

  let error_data = json!({
      "type": client_error.error_code,
      "code": status_code.as_u16(),
      "message": client_error.message,
      "details": client_error.details
  });

  let api_response = ApiResponse::error(uuid, error_data);

  ProcessedResponse { status_code, body: api_response.to_json(), client_error: Some(client_error) }
}

/// Process error response from response body
fn process_body_error_response(
  uuid: Uuid,
  status_code: StatusCode,
  error_data: Value,
) -> ProcessedResponse {
  let api_response = ApiResponse::error(uuid, error_data);
  ProcessedResponse { status_code, body: api_response.to_json(), client_error: None }
}

/// Process unknown error response
fn process_unknown_error_response(uuid: Uuid, status_code: StatusCode) -> ProcessedResponse {
  let error_data = json!({
      "type": "UNKNOWN_ERROR",
      "code": status_code.as_u16(),
      "message": "An unexpected error occurred"
  });

  let api_response = ApiResponse::error(uuid, error_data);
  ProcessedResponse { status_code, body: api_response.to_json(), client_error: None }
}

/// Log the request based on success/error status
async fn log_request_response(
  uri: Uri,
  req_method: Method,
  req_stamp: ReqStamp,
  ctx: Option<jd_core::ctx::Ctx>,
  request_body: Option<Value>,
  processed: &ProcessedResponse,
  web_error: Option<&Error>,
) {
  let request_log =
    LogRequest { uri, method: req_method, stamp: req_stamp, ctx, body: request_body };

  let log_response = LogResponse {
    body: Some(processed.body.clone()),
    error: web_error.cloned(),
    client_error: processed.client_error.clone(),
  };

  let log_entry = LogEntry::new(request_log, log_response);

  if let Err(e) = log_request(log_entry).await {
    error!("Failed to log request: {}", e);
  }
}

/// Log appropriate message based on response type
fn log_response_message(req_method: &Method, uri: &Uri, processed: &ProcessedResponse) {
  if processed.client_error.is_some()
    || processed.status_code.is_client_error()
    || processed.status_code.is_server_error()
  {
    if let Some(ref client_error) = processed.client_error {
      error!(
        "Request failed: {} {} - Status: {} - Error: {} - Details: {:?}",
        req_method, uri, processed.status_code, client_error.error_code, client_error.details
      );
    } else {
      error!("Request failed: {} {} - Status: {}", req_method, uri, processed.status_code);
    }
  } else {
    info!("Request completed successfully: {} - {}", req_method, uri);
  }
}

pub async fn mw_map_response(
  ctx: Result<CtxW>,
  uri: Uri,
  req_method: Method,
  req_stamp: ReqStamp,
  res: Response,
) -> Response {
  let ctx = ctx.map(|ctx| ctx.0).ok();
  let ReqStamp { uuid, .. } = req_stamp;

  let (parts, body) = res.into_parts();
  let extension = parts.extensions.clone();
  let web_error = extension.get::<Error>();
  let request_body = extension.get::<Value>().cloned();

  let processed = if parts.status.is_success() {
    // Handle successful response
    let data = extract_response_body(body).await;
    process_success_response(uuid, data)
  } else if let Some(err) = web_error {
    // Handle web error
    process_web_error_response(uuid, err)
  } else {
    // Handle other errors by parsing response body
    let data = extract_response_body(body).await;
    if data != Value::Null {
      process_body_error_response(uuid, parts.status, data)
    } else {
      process_unknown_error_response(uuid, parts.status)
    }
  };

  // Log the response message
  log_response_message(&req_method, &uri, &processed);

  // Log the request details
  log_request_response(uri, req_method, req_stamp, ctx, request_body, &processed, web_error).await;

  // Return the processed response
  (processed.status_code, Json(processed.body)).into_response()
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_api_response_success() {
    let uuid = Uuid::new_v4();
    let data = json!({"result": "test"});
    let response = ApiResponse::success(uuid, data.clone());

    assert_eq!(response.status, 0);
    assert_eq!(response.response_type, "success");
    assert_eq!(response.data, Some(data));
    assert!(response.error.is_none());
  }

  #[test]
  fn test_api_response_error() {
    let uuid = Uuid::new_v4();
    let error_data = json!({"message": "test error"});
    let response = ApiResponse::error(uuid, error_data.clone());

    assert_eq!(response.status, 1);
    assert_eq!(response.response_type, "error");
    assert!(response.data.is_none());
    assert_eq!(response.error, Some(error_data));
  }
}
