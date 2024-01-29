use std::convert::Infallible;

use axum::{
  extract::FromRequestParts,
  http::{request::Parts, HeaderMap},
};

use crate::error::AppError;

/// Rejects if `X-Auth-Key` is invalid or missing from headers
#[derive(aide::OperationIo)]
#[aide(input)]
pub struct ApiKey(pub Result<(), AppError>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for ApiKey
where
  S: Send + Sync,
{
  type Rejection = Infallible;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    // safe because of Err type is Infallible
    let headers = unsafe {
      HeaderMap::from_request_parts(parts, state)
        .await
        .unwrap_unchecked()
    };
    let value = headers
      .get("X-Auth-Key")
      .ok_or(AppError::AuthKeyMissing)
      .and_then(|v| match v.as_bytes() {
        b"CORRECT_API_KEY" => Ok(()),
        b"CORRECT_API_KEY_BUT_NO_ACCESS" => Err(AppError::AuthKeyNoAccess),
        _ => Err(AppError::AuthKeyInvalid),
      });

    Ok(Self(value))
  }
}
