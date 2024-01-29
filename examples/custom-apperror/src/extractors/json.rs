use axum::extract::{FromRequest, Request};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::{AppError, AppErrorCode};

use super::set_inferred_response;

pub struct JsonValidate<T>(pub T);

#[axum::async_trait]
impl<T, S> FromRequest<S> for JsonValidate<T>
where
  T: DeserializeOwned + Validate + schemars::JsonSchema,
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let value: T = axum::Json::from_request(req, state).await?.0;
    value.validate()?;
    Ok(Self(value))
  }
}

impl<T: JsonSchema> aide::OperationInput for JsonValidate<T> {
  fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
    [AppErrorCode::Validation, AppErrorCode::JsonRejection]
      .into_iter()
      .for_each(|status| set_inferred_response(ctx, operation, status));

    // <axum::Json<AppErrorOutput> as aide::OperationInput>::operation_input(ctx, operation);
    <axum::Json<T> as aide::OperationInput>::operation_input(ctx, operation)
  }
}
