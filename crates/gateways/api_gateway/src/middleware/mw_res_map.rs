use crate::Result;
use crate::{error::Error, log::log_request};
use axum::{
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};

use jd_utils::time::{self, format_time, now_utc};
use serde_json::{json, to_value};
use std::sync::Arc;
use tracing::debug;

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
    let client_error = client_status_error.clone().unzip().1;

    let _ = log_request(uri, req_method, req_stamp, ctx, web_error, client_error).await;

    let error_response = client_status_error.as_ref().map(|(status_code, client_error)| {
        let client_error = to_value(client_error).ok();
        let message = client_error.as_ref().and_then(|v| v.get("message"));
        let detail = client_error.as_ref().and_then(|v| v.get("detail"));

        let client_error_body = json!({
            "id": uuid.to_string(),
            "status": "error",
            "timestamp": format_time(now_utc()),
            "error": {
                "message": message,
                "data" : {
                    "type": client_error.as_ref(),
                    "code": status_code.as_u16()
                },
                "detail": detail
            }
        });

        debug!("CLIENT ERROR BODY:\n{client_error_body}");

        (*status_code, Json(client_error_body)).into_response()
    });

    debug!("\n");

    // TODO: fix response
    error_response.unwrap_or_else(|| Response::from_parts(parts, body))
}
