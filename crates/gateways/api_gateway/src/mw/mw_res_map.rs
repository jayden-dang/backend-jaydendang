use crate::{error::Error, Result};
use axum::{
    body::to_bytes,
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::{json, to_value, Value};
use std::sync::Arc;
use tracing::{debug, error, warn};
use uuid::Uuid;

pub async fn mw_map_response(uri: Uri, req_method: Method, res: Response) -> Response {
    let uuid = Uuid::new_v4();
    let status = res.status();

    // Handle error cases
    let web_error = res.extensions().get::<Arc<Error>>().map(Arc::as_ref);
    let client_status_error = web_error.map(|e| e.client_status_and_error());

    match client_status_error {
        Some((status_code, client_error)) => {
            let client_error = to_value(client_error).ok();
            let message = client_error.as_ref().and_then(|v| v.get("message"));
            let details = client_error.as_ref().and_then(|v| v.get("details"));

            let error_body = json!({
                "req_id": uuid.to_string(),
                "data": {
                    "details": details,
                    "message": message,
                },
                "status": 0,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            warn!("Error Response: {} - {}", status_code, uri);
            let _ = log_request(uuid, uri, req_method, error_body.clone(), 0).await;
            (status_code, Json(error_body)).into_response()
        }
        None => {
            // Handle successful responses
            let body = match to_bytes(res.into_body(), usize::MAX).await {
                Ok(body) => body,
                Err(e) => {
                    error!("Failed to read response body: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "req_id": uuid.to_string(),
                            "status": 0,
                            "data": {
                                "message": "Failed to process response",
                                "details": e.to_string()
                            },
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }))
                    ).into_response();
                }
            };

            let body_string = String::from_utf8(body.to_vec()).unwrap_or_default();
            let data: Value = match serde_json::from_str(&body_string) {
                Ok(data) => data,
                Err(_) => {
                    // If not JSON, return as plain text
                    json!({
                        "content": body_string,
                        "content_type": "text/plain"
                    })
                }
            };

            let json_response = json!({
                "req_id": uuid.to_string(),
                "status": 1,
                "data": data,
                "metadata": {
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "path": uri.path(),
                    "method": req_method.to_string()
                }
            });

            debug!("Success Response: {} - {}", status, uri);
            let _ = log_request(uuid, uri, req_method, json_response.clone(), 1).await;
            (status, Json(json_response)).into_response()
        }
    }
}

async fn log_request(uuid: Uuid, uri: Uri, req_method: Method, error_data: Value, status: u8) -> Result<()> {
    let log = RequestLogLine {
        uuid: uuid.to_string(),
        http_method: req_method.to_string(),
        http_path: uri.to_string(),
        error_data,
        status,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if status == 0 {
        error!("Request Log: {}", json!(log));
    } else {
        debug!("Request Log: {}", json!(log));
    }
    Ok(())
}

#[derive(Serialize)]
struct RequestLogLine {
    uuid: String,
    http_path: String,
    http_method: String,
    error_data: Value,
    status: u8,
    timestamp: String,
}
