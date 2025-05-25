use std::sync::Arc;

use crate::{error::Error, Result};
use axum::{extract::State, Json};
use jd_contracts::user::dto::CreateUserRequest;
use jd_core::{base, AppState};

use super::{record::UserRecord, UsersDmc};
