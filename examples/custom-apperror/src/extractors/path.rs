use axum::{extract::FromRequestParts, http::request::Parts};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::{AppError, AppErrorCode};

use super::set_inferred_response;

/// Extract Path variables, and apply validation checks
pub struct PathValidate<T>(pub T);

#[axum::async_trait]
impl<T, S> FromRequestParts<S> for PathValidate<T>
where
  T: DeserializeOwned + Validate + schemars::JsonSchema + Send,
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let value: T = axum::extract::Path::from_request_parts(parts, state)
      .await?
      .0;
    value.validate()?;
    Ok(Self(value))
  }
}

impl<T: JsonSchema> aide::OperationInput for PathValidate<T> {
  fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
    [AppErrorCode::Validation, AppErrorCode::PathRejection]
      .into_iter()
      .for_each(|status| set_inferred_response(ctx, operation, status));
    // <axum::Json<AppErrorOutput> as aide::OperationInput>::operation_input(ctx, operation);
    <axum::extract::Path<T> as aide::OperationInput>::operation_input(ctx, operation)
  }
  // Not sure why `inferred_early_responses` is never called :(
  // so it will not be used
  // fn inferred_early_responses(
  //   ctx: &mut aide::gen::GenContext,
  //   operation: &mut aide::openapi::Operation,
  // ) -> Vec<(Option<u16>, aide::openapi::Response)> {
  //   println!("hmmm path_val early");
  //   <axum::extract::Path<T> as aide::OperationInput>::inferred_early_responses(ctx, operation)
  // }
}
