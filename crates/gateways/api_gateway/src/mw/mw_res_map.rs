use crate::{error::Error, log::log_request};
use axum::{
    body::to_bytes,
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use jd_utils::time::format_time;
use serde_json::{json, to_value, Value};
use std::sync::Arc;

use super::mw_res_timestamp::ReqStamp;

pub async fn mw_map_response(uri: Uri, req_method: Method, req_stamp: ReqStamp, res: Response) -> Response {
    let status = res.status();
    let headers = get_request_headers(&res);

    // Handle error cases
    let web_error = res.extensions().get::<Arc<Error>>().map(Arc::as_ref);
    let client_status_error = web_error.map(|e| e.client_status_and_error());
    let ReqStamp { uuid, time_in } = req_stamp;
    let request_time = format_time(time_in);

    match client_status_error {
        Some((status_code, client_error)) => {
            let client_error = to_value(client_error).ok();
            let message = client_error.as_ref().and_then(|v| v.get("message"));
            let details = client_error.as_ref().and_then(|v| v.get("details"));

            // Client response - minimal information
            let client_response = json!({
                "request_id": uuid.to_string(),
                "status": 0,
                "data": {
                    "message": message,
                    "details": details
                },
                "meta": {
                    "timestamp": request_time,
                }
            });

            let _ = log_request(uri, req_method, req_stamp, None).await;
            (status_code, Json(client_response)).into_response()
        }
        None => {
            // Handle successful responses
            let body = match to_bytes(res.into_body(), usize::MAX).await {
                Ok(body) => body,
                Err(e) => {
                    let client_response = json!({
                        "request_id": uuid.to_string(),
                        "status": 0,
                        "data": {
                            "message": "Failed to process response",
                            "details": e.to_string()
                        },
                        "meta": {
                            "timestamp": request_time,
                        }
                    });

                    let _ = log_request(uri, req_method, req_stamp, None).await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(client_response)).into_response();
                }
            };

            let body_string = String::from_utf8(body.to_vec()).unwrap_or_default();
            let content_type = headers
                .get("content-type")
                .and_then(|v| v.as_str())
                .unwrap_or("application/json");
            let pagination: Option<Value> = None; // TODO: Add pagination information
            let data: Value = match serde_json::from_str::<Value>(&body_string) {
                Ok(data) => {
                    // If the response already has the expected format, use it directly
                    if data.is_object() && data.get("data").is_some() && data.get("meta").is_some() {
                        data
                    } else {
                        // Otherwise wrap it in the standard format
                        json!({
                            "data": data,
                            "meta": {
                                "timestamp": request_time,
                                "content_type": content_type,
                                "pagination": pagination
                            }
                        })
                    }
                }
                Err(_) => {
                    // If not JSON, return as plain text
                    json!({
                        "data": {
                            "content": body_string,
                            "content_type": "text/plain"
                        },
                        "meta": {
                            "timestamp": request_time,
                            "content_type": "text/plain",
                            "pagination": pagination
                        }
                    })
                }
            };

            // Client response - minimal information
            let client_response = json!({
                "request_id": uuid.to_string(),
                "status": 1,
                "data": data["data"],
                "meta": data["meta"]
            });

            let server_log = json!({
                "data" : data,
                "headers": headers,
                "response": {
                    "size_bytes" : body_string.len()
                }
            });

            // Server log - detailed information
            let _ = log_request(uri, req_method, req_stamp, Some(server_log)).await;
            (status, Json(client_response)).into_response()
        }
    }
}

fn get_request_headers(res: &Response) -> Value {
    let important_headers = [
        "content-type",
        "content-length",
        "user-agent",
        "accept",
        "accept-encoding",
        "accept-language",
        "connection",
        "host",
        "origin",
        "referer",
    ];

    let headers: std::collections::HashMap<String, String> = res
        .headers()
        .iter()
        .filter(|(name, _)| important_headers.contains(&name.as_str().to_lowercase().as_str()))
        .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
        .collect();
    json!(headers)
}
