use crate::Result;
use crate::error::Error;
use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::Request;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use jd_utils::time::now_utc;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ReqStamp {
  pub uuid: Uuid,
  pub time_in: OffsetDateTime,
}

pub async fn mw_req_stamp_resolver(mut req: Request<Body>, next: Next) -> Result<Response> {
  let time_in = now_utc();
  let uuid = Uuid::new_v4();

  req.extensions_mut().insert(ReqStamp { uuid, time_in });

  Ok(next.run(req).await)
}

// region:    --- ReqStamp Extractor
impl<S: Send + Sync> FromRequestParts<S> for ReqStamp {
  type Rejection = Error;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
    parts
      .extensions
      .get::<ReqStamp>()
      .cloned()
      .ok_or(Error::ReqStampNotInReqExt)
  }
}
// endregion: --- ReqStamp Extractor
