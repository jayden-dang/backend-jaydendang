use std::{fmt::Display, sync::Arc};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, Clone, From)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    BadRequest(String),
    // -- Login
    LoginFail,
    // -- CtxExtError
    EntityNotFound {
        entity: &'static str,
        id: i64,
    },

    #[from]
    CoreError(#[serde_as(as = "DisplayFromStr")] Arc<jd_core::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

// region:    --- Axum IntoResponse
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - model::Error {self:?}", "INTO_RES");

        // Get the status code and client error
        let (status_code, _) = self.client_status_and_error();

        // Create a placeholder Axum response
        let mut response = status_code.into_response();

        // Insert the Error into the response
        response.extensions_mut().insert(self);

        response
    }
}
// endregion: --- Axum IntoResponse

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use Error::*;

        match self {
            // -- Login/Auth
            LoginFail => (StatusCode::UNAUTHORIZED, ClientError::LOGIN_FAIL),
            BadRequest(_) => (StatusCode::BAD_REQUEST, ClientError::SERVICE_ERROR),
            EntityNotFound { entity, id } => (StatusCode::NOT_FOUND, ClientError::EntityNotFound { entity, id: *id }),
            CoreError(core_err) => match core_err.as_ref() {
                jd_core::Error::CantCreateModelManagerProvider(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR)
                }
                jd_core::Error::ListLimitOverMax { max, actual } => (
                    StatusCode::BAD_REQUEST,
                    ClientError::ListLimitOverMax {
                        max: *max,
                        actual: *actual,
                    },
                ),
                jd_core::Error::UniqueViolation { table, constraint } => (
                    StatusCode::CONFLICT,
                    ClientError::UniqueViolation {
                        table: table.clone(),
                        constraint: constraint.clone(),
                    },
                ),
                jd_core::Error::CountFail => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
                jd_core::Error::EntityNotFound { entity, id } => {
                    (StatusCode::NOT_FOUND, ClientError::EntityNotFound { entity, id: *id })
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, ClientError::SERVICE_ERROR),
            },
        }
    }
}

#[derive(Debug, strum_macros::AsRefStr, Serialize, Clone)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    SERVICE_ERROR,
    EntityNotFound { entity: &'static str, id: i64 },
    ListLimitOverMax { max: i64, actual: i64 },
    UniqueViolation { table: String, constraint: String },
}
// endregion: --- Client Error
