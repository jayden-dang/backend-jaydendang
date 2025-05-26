use crate::Result;
use crate::{error::Error, log::log_request};
use axum::body::to_bytes;
use axum::{
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};

use jd_utils::time::{format_time, now_utc};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use super::{mw_auth::CtxW, mw_res_timestamp::ReqStamp};
use crate::error::RequestContext;

pub async fn mw_map_response(
    ctx: Result<CtxW>,
    uri: Uri,
    req_method: Method,
    req_stamp: ReqStamp,
    res: Response,
) -> Response {
    let ctx = ctx.map(|ctx| ctx.0).ok();
    let ReqStamp { uuid, time_in: _ } = req_stamp;

    let (parts, body) = res.into_parts();
    let extension = parts.extensions.clone();
    let web_error = extension.get::<Error>();

    // Get request body from extension if available
    let request_body = extension.get::<Value>().cloned();

    // Check if the status code indicates success
    let is_success = parts.status.is_success();

    if is_success {
        let body = to_bytes(body, usize::MAX).await.unwrap_or_default();
        let body_string = String::from_utf8(body.to_vec()).unwrap_or_default();
        let data: Value = serde_json::from_str(&body_string).unwrap_or(Value::Null);

        let success_body = json!({
            "id": uuid.to_string(),
            "status": 0,
            "type": "success",
            "data": data,
            "meta": {
                "timestamp": format_time(now_utc()),
            }
        });

        // Log request details
        info!("Request Completed Successfully: {} - {}", req_method, uri);
        let _ = log_request(
            uri,
            req_method,
            req_stamp,
            ctx,
            web_error,
            None,
            request_body,
            Some(success_body.clone()),
        )
        .await;
        (parts.status, Json(success_body)).into_response()
    } else {
        // If we have a web_error, use its status and error type
        if let Some(err) = web_error {
            let request_context = RequestContext::default();
            let (status_code, client_error) = err.client_status_and_error(&request_context);

            let error_body = json!({
                "id": uuid.to_string(),
                "status": 1,
                "type": "error",
                "meta": {
                    "timestamp": format_time(now_utc()),
                },
                "error": {
                    "type": client_error.error_code,
                    "code": status_code.as_u16(),
                    "message": client_error.message,
                    "details": client_error.details
                }
            });

            error!(
                "Request failed: {} {} - Status: {} - Error: {} - Details: {:?}",
                req_method, uri, status_code, client_error.error_code, client_error.details
            );

            // Log request details with client error
            let _ = log_request(
                uri,
                req_method,
                req_stamp,
                ctx,
                web_error,
                Some(client_error),
                request_body,
                Some(error_body.clone()),
            )
            .await;
            return (status_code, Json(error_body)).into_response();
        }

        // If we have a response body, try to parse it as an error
        let body = to_bytes(body, usize::MAX).await.unwrap_or_default();
        let body_string = String::from_utf8(body.to_vec()).unwrap_or_default();
        if let Ok(error_data) = serde_json::from_str::<Value>(&body_string) {
            // If not a conflict error, use the original error data
            let error_body = json!({
                "id": uuid.to_string(),
                "status": 1,
                "type": "error",
                "meta": {
                    "timestamp": format_time(now_utc()),
                },
                "error": error_data
            });

            error!(
                "Request failed with error from response body: {} {} - Status: {}",
                req_method, uri, parts.status
            );

            // Log request details with error from response body
            let _ = log_request(
                uri,
                req_method,
                req_stamp,
                ctx,
                web_error,
                None,
                request_body,
                Some(error_body.clone()),
            )
            .await;
            return (parts.status, Json(error_body)).into_response();
        }

        // Fallback for unknown errors
        let error_body = json!({
            "id": uuid.to_string(),
            "status": 1,
            "type": "error",
            "meta": {
                "timestamp": format_time(now_utc()),
            },
            "error": {
                "type": "UNKNOWN_ERROR",
                "code": parts.status.as_u16(),
                "message": "An unexpected error occurred"
            }
        });

        // Log request details with unknown error
        let _ = log_request(
            uri,
            req_method,
            req_stamp,
            ctx,
            web_error,
            None,
            request_body,
            Some(error_body.clone()),
        )
        .await;
        (parts.status, Json(error_body)).into_response()
    }
}
