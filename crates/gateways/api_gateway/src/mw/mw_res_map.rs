use crate::{error::Error, log::log_request};
use axum::{
    body::to_bytes,
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, to_value, Value};
use std::sync::Arc;
use uuid::Uuid;

pub async fn mw_map_response(uri: Uri, req_method: Method, res: Response) -> Response {
    eprintln!("->> {:<12} - mw_map_response - {} {}", "MIDDLEWARE", req_method, uri);
    let start_time = std::time::Instant::now();
    let uuid = Uuid::new_v4();
    let status = res.status();
    let headers = get_request_headers(&res);
    let request_time = chrono::Utc::now();

    // Handle error cases
    let web_error = res.extensions().get::<Arc<Error>>().map(Arc::as_ref);
    let client_status_error = web_error.map(|e| e.client_status_and_error());

    match client_status_error {
        Some((status_code, client_error)) => {
            eprintln!("->> {:<12} - Error Response", "MIDDLEWARE");
            let client_error = to_value(client_error).ok();
            let message = client_error.as_ref().and_then(|v| v.get("message"));
            let details = client_error.as_ref().and_then(|v| v.get("details"));
            let end_time = std::time::Instant::now();

            // Client response - minimal information
            let client_response = json!({
                "request_id": uuid.to_string(),
                "status": 0,
                "data": {
                    "message": message,
                    "details": details
                },
                "meta": {
                    "timestamp": request_time.to_rfc3339()
                }
            });

            // Server log - detailed information
            let server_log = json!({
                "log_type": "request",
                "timestamp": request_time.to_rfc3339(),
                "request_id": uuid.to_string(),
                "request": {
                    "method": req_method.to_string(),
                    "path": uri.path().to_string(),
                    "query": uri.query().map(|q| q.to_string()),
                    "headers": headers,
                    "client_ip": None::<String> // TODO: Add client IP
                },
                "response": {
                    "status_code": status_code.as_u16(),
                    "time_ms": end_time.duration_since(start_time).as_millis(),
                    "size_bytes": 0
                },
                "server": {
                    "version": env!("CARGO_PKG_VERSION"),
                    "environment": std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
                    "hostname": hostname::get().ok().and_then(|h| h.into_string().ok()),
                    "started_at": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }
            });

            eprintln!("->> {:<12} - Server Log: {}", "MIDDLEWARE", server_log);
            let _ = log_request(uuid, uri, req_method, server_log, status_code.as_u16() as u8).await;
            (status_code, Json(client_response)).into_response()
        }
        None => {
            eprintln!("->> {:<12} - Success Response", "MIDDLEWARE");
            // Handle successful responses
            let body = match to_bytes(res.into_body(), usize::MAX).await {
                Ok(body) => body,
                Err(e) => {
                    eprintln!("->> {:<12} - Failed to read body: {}", "MIDDLEWARE", e);
                    let end_time = std::time::Instant::now();

                    // Client response - minimal information
                    let client_response = json!({
                        "request_id": uuid.to_string(),
                        "status": 0,
                        "data": {
                            "message": "Failed to process response",
                            "details": e.to_string()
                        },
                        "meta": {
                            "timestamp": request_time.to_rfc3339()
                        }
                    });

                    // Server log - detailed information
                    let server_log = json!({
                        "log_type": "request",
                        "timestamp": request_time.to_rfc3339(),
                        "request_id": uuid.to_string(),
                        "request": {
                            "method": req_method.to_string(),
                            "path": uri.path().to_string(),
                            "query": uri.query().map(|q| q.to_string()),
                            "headers": headers,
                            "client_ip": None::<String> // TODO: Add client IP
                        },
                        "response": {
                            "status_code": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                            "time_ms": end_time.duration_since(start_time).as_millis(),
                            "size_bytes": 0
                        },
                        "server": {
                            "version": env!("CARGO_PKG_VERSION"),
                            "environment": std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
                            "hostname": hostname::get().ok().and_then(|h| h.into_string().ok()),
                            "started_at": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        }
                    });

                    eprintln!("->> {:<12} - Server Log: {}", "MIDDLEWARE", server_log);
                    let _ = log_request(
                        uuid,
                        uri,
                        req_method,
                        server_log,
                        StatusCode::INTERNAL_SERVER_ERROR.as_u16() as u8,
                    )
                    .await;
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
                                "timestamp": request_time.to_rfc3339(),
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
                            "timestamp": request_time.to_rfc3339(),
                            "content_type": "text/plain",
                            "pagination": pagination
                        }
                    })
                }
            };

            let end_time = std::time::Instant::now();

            // Client response - minimal information
            let client_response = json!({
                "request_id": uuid.to_string(),
                "status": 1,
                "data": data["data"],
                "meta": data["meta"]
            });

            // Server log - detailed information
            let server_log = json!({
                "log_type": "request",
                "timestamp": request_time.to_rfc3339(),
                "request_id": uuid.to_string(),
                "request": {
                    "method": req_method.to_string(),
                    "path": uri.path().to_string(),
                    "query": uri.query().map(|q| q.to_string()),
                    "headers": headers,
                    "client_ip": None::<String> // TODO: Add client IP
                },
                "response": {
                    "status_code": status.as_u16(),
                    "time_ms": end_time.duration_since(start_time).as_millis(),
                    "size_bytes": body_string.len()
                },
                "server": {
                    "version": env!("CARGO_PKG_VERSION"),
                    "environment": std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
                    "hostname": hostname::get().ok().and_then(|h| h.into_string().ok()),
                    "started_at": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }
            });

            eprintln!("->> {:<12} - Server Log: {}", "MIDDLEWARE", server_log);
            let _ = log_request(uuid, uri, req_method, server_log, status.as_u16() as u8).await;
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
