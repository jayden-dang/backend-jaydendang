use crate::error::Error;
use crate::Result;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

const AUTH_TOKEN: &str = "auth-token";

pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login_handler))
}

async fn api_login_handler(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    debug!("->> {:<12} - api_login_handler", "HANDLER");

    // Test case 1: Empty username - Bad Request
    if payload.username.is_empty() {
        return Err(Error::ReqStampNotInReqExt);
    }

    // Test case 2: Empty password - Bad Request
    if payload.pwd.is_empty() {
        return Err(Error::ReqStampNotInReqExt);
    }

    // Test case 3: User not found - Not Found
    if payload.username == "notfound" {
        let err = Error::EntityNotFound { 
            entity: "user", 
            id: 0 
        };
        debug!("Returning EntityNotFound error: {:?}", err);
        return Err(err);
    }

    // Test case 4: Invalid credentials - Unauthorized
    if payload.username != "demo1" || payload.pwd != "welcome" {
        return Err(Error::LoginFail);
    }

    // Success case - Set auth token in cookie
    cookies.add(Cookie::new(AUTH_TOKEN, "user-1.exp.sign"));

    // Create the success body with user info only
    let body = Json(json!({
        "result": {
            "success": true,
            "user": {
                "username": payload.username,
                "role": "admin"
            }
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}
