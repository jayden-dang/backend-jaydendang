use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
use serde::Serialize;
use serde_with::serde_as;

use crate::middleware::{self};

#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, From, Clone)]
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
    ReqStampNotInReqExt,

    #[from]
    CtxExt(middleware::mw_auth::CtxExtError),

    #[from]
    CoreError(Arc<jd_core::Error>),
}

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

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate

/// From the root error to the http status code and ClientError
impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use Error::*;

        match self {
            // -- Login/Auth
            LoginFail => (StatusCode::UNAUTHORIZED, ClientError::LOGIN_FAIL),
            BadRequest(_) => (StatusCode::BAD_REQUEST, ClientError::SERVICE_ERROR),
            CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),
            EntityNotFound { entity, id } => (StatusCode::NOT_FOUND, ClientError::EntityNotFound { entity, id: *id }),
            ReqStampNotInReqExt => (StatusCode::BAD_REQUEST, ClientError::SERVICE_ERROR),
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
