use aide::{
  openapi::{Parameter, ParameterData},
  operation::add_parameters,
};
use axum::{
  extract::FromRequestParts,
  http::{request::Parts, HeaderMap},
};
use indexmap::IndexMap;

use crate::error::{AppError, AppErrorCode};

/// Rejects if `X-Auth-Key` is invalid or missing from headers
pub struct ApiKey();

#[axum::async_trait]
impl<S> FromRequestParts<S> for ApiKey
where
  S: Send + Sync,
{
  type Rejection = AppError;

  async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let headers = unsafe {
      HeaderMap::from_request_parts(parts, state)
        .await
        .unwrap_unchecked()
    };
    let value = headers.get("X-Auth-Key").ok_or(AppError::AuthKeyMissing)?;
    match value.as_bytes() {
      b"CORRECT_API_KEY" => (),
      b"CORRECT_API_KEY_BUT_NO_ACCESS" => return Err(AppError::AuthKeyNoAccess),
      _ => return Err(AppError::AuthKeyInvalid),
    }

    Ok(Self())
  }
}

impl aide::OperationInput for ApiKey {
  fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
    add_parameters(
      ctx,
      operation,
      [Parameter::Header {
        parameter_data: ParameterData {
          name: "X-Auth-Key".to_owned(),
          description: None,
          required: true,
          deprecated: None,
          format: aide::openapi::ParameterSchemaOrContent::Schema(aide::openapi::SchemaObject {
            json_schema: schemars::schema::Schema::Object(schemars::schema_for!(String).schema),
            external_docs: None,
            example: None,
          }),
          example: None,
          examples: IndexMap::new(),
          explode: None,
          extensions: IndexMap::new(),
        },
        style: aide::openapi::HeaderStyle::Simple,
      }],
    );

    [
      AppErrorCode::AuthKeyInvalid,
      AppErrorCode::AuthKeyMissing,
      AppErrorCode::AuthKeyNoAccess,
    ]
    .into_iter()
    .for_each(|status| super::set_inferred_response(ctx, operation, status));
  }
}
