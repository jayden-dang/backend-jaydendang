use axum::{
    routing::{get, post},
    Router,
};
use jd_contracts::user::api::{CREATE_USER_PATH, GET_USER_PATH};
use jd_core::AppState;
