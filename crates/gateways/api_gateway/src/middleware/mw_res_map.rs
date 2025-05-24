use crate::Result;
use crate::{error::Error, log::log_request};
use axum::body::to_bytes;
use axum::{
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};

use jd_utils::time::{self, format_time, now_utc};
use serde_json::{json, to_value, Value};
use std::sync::Arc;
use tracing::{debug, error};

use super::{mw_auth::CtxW, mw_res_timestamp::ReqStamp};

pub async fn mw_map_response(
    ctx: Result<CtxW>,
    uri: Uri,
    req_method: Method,
    req_stamp: ReqStamp,
    res: Response,
) -> Response {
    debug!("->> {:<12} - mw_map_response", "RES_MAPPER");
    let ctx = ctx.map(|ctx| ctx.0).ok();
    let ReqStamp { uuid, time_in } = req_stamp;

    let (parts, body) = res.into_parts();
    let extension = parts.extensions.clone();
    let web_error = extension.get::<Arc<Error>>().map(Arc::as_ref);
    let client_status_error = web_error.map(|e| e.client_status_and_error());
    let client_status_error_clone = client_status_error.clone();

    // Check if the status code indicates success
    let is_success = parts.status.is_success();

    if is_success {
        debug!("HANDLING SUCCESS CASE: status={}", parts.status);
        let body = to_bytes(body, usize::MAX).await.unwrap_or_default();
        let body_string = String::from_utf8(body.to_vec()).unwrap_or_default();
        let data: Value = serde_json::from_str(&body_string).unwrap_or(Value::Null);

        let success_body = json!({
            "id": uuid.to_string(),
            "status": "success",
            "data": data,
            "meta": {
                "timestamp": format_time(now_utc()),
            }
        });

        let client_error = client_status_error_clone.unzip().1;
        let _ = log_request(uri, req_method, req_stamp, ctx, web_error, client_error).await;
        (parts.status, Json(success_body)).into_response()
    } else {
        debug!("HANDLING ERROR CASE: status={}", parts.status);
        let (status_code, client_error) = client_status_error
            .map(|(code, err)| (code, Some(err)))
            .unwrap_or((parts.status, None));

        let error_body = json!({
            "id": uuid.to_string(),
            "status": "error",
            "timestamp": format_time(now_utc()),
            "error": {
                "type": client_error.map(|e| e.as_ref().to_string()).unwrap_or("UNKNOWN_ERROR".to_string()),
                "code": status_code.as_u16()
            }
        });
        error!("CLIENT ERROR BODY: \n {error_body}");
        (status_code, Json(error_body)).into_response()
    }
}
