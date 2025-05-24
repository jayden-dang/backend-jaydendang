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

    // Test case 1: Empty username
    if payload.username.is_empty() {
        return Err(Error::LoginFail);
    }

    // Test case 2: Empty password
    if payload.pwd.is_empty() {
        return Err(Error::LoginFail);
    }

    // Test case 3: Invalid credentials
    if payload.username != "demo1" || payload.pwd != "welcome" {
        return Err(Error::LoginFail);
    }

    // Success case
    cookies.add(Cookie::new(AUTH_TOKEN, "user-1.exp.sign"));

    // Create the success body with more detailed response
    let body = Json(json!({
        "result": {
            "success": true,
            "user": {
                "username": payload.username,
                "role": "admin"
            },
            "token": "user-1.exp.sign"
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}
