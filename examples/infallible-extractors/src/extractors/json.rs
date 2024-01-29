use std::convert::Infallible;

use axum::extract::{FromRequest, Request};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::AppError;

/// Deserialize request body into json, and apply validation checks
#[derive(aide::OperationIo)]
#[aide(input)]
pub struct JsonValidate<T>(pub Result<T, AppError>);

#[axum::async_trait]
impl<T, S> FromRequest<S> for JsonValidate<T>
where
  T: DeserializeOwned + Validate + JsonSchema,
  S: Send + Sync,
{
  type Rejection = Infallible;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let value = axum::Json::from_request(req, state)
      .await
      .map_err(|e| AppError::from(e))
      .and_then(|v: axum::Json<T>| v.0.validate().map(|_| v.0).map_err(|e| AppError::from(e)));
    Ok(Self(value))
  }
}
