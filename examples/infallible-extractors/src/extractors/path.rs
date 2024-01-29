use std::convert::Infallible;

use axum::{extract::FromRequestParts, http::request::Parts};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;

/// Extract Path variables, and apply validation checks
#[derive(aide::OperationIo)]
#[aide(input)]
pub struct PathValidate<T>(pub Result<T, AppError>);

#[axum::async_trait]
impl<T, S> FromRequestParts<S> for PathValidate<T>
where
  T: DeserializeOwned + Validate + JsonSchema + Send,
  S: Send + Sync,
{
  type Rejection = Infallible;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    use axum::extract::Path;
    let value = Path::from_request_parts(parts, state)
      .await
      .map_err(|e| AppError::from(e))
      .and_then(|v: Path<T>| v.0.validate().map(|_| v.0).map_err(|e| AppError::from(e)));
    Ok(Self(value))
  }
}
