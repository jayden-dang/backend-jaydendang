use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
use serde::Serialize;
use serde_with::serde_as;

use crate::middleware::{self};

#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr, Clone, From)]
#[serde(tag = "type", content = "data")]
pub enum Error {
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

// region:    --- Client Error

/// From the root error to the http status code and ClientError
impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use Error::*;

        match self {
            // -- Login/Auth
            LoginFail => (StatusCode::UNAUTHORIZED, ClientError::LOGIN_FAIL),
            CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),
            EntityNotFound { entity, id } => (StatusCode::NOT_FOUND, ClientError::EntityNotFound { entity, id: *id }),
            ReqStampNotInReqExt => (StatusCode::BAD_REQUEST, ClientError::SERVICE_ERROR),
            // -- Fallback
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
}
// endregion: --- Client Error
