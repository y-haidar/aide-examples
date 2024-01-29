use crate::error::{AppErrorCode, AppErrorOutput};
use schemars::JsonSchema;

pub mod auth;
pub mod json;
pub mod path;

pub use auth::*;
pub use json::*;
pub use path::*;

fn set_inferred_response(
  ctx: &mut aide::gen::GenContext,
  operation: &mut aide::openapi::Operation,
  status: AppErrorCode,
) {
  use aide::openapi::{ReferenceOr, StatusCode};

  let example = unsafe { serde_json::to_value(status.create_example()).unwrap_unchecked() };
  let description = status.description();
  let res = app_error_operation_response::<AppErrorOutput>(
    ctx,
    operation,
    Some(example),
    description.to_owned(),
  );
  let status = Some(status.status_code_as_u16());

  if operation.responses.is_none() {
    operation.responses = Some(Default::default());
  }

  let responses = operation.responses.as_mut().unwrap();

  match status {
    Some(status) => {
      if responses.responses.contains_key(&StatusCode::Code(status)) {
        ctx.error(aide::Error::InferredResponseConflict(status));
      } else {
        responses
          .responses
          .insert(StatusCode::Code(status), ReferenceOr::Item(res));
      }
    }
    None => {
      if responses.default.is_some() {
        ctx.error(aide::Error::InferredDefaultResponseConflict);
      } else {
        responses.default = Some(ReferenceOr::Item(res));
      }
    }
  }
}

fn app_error_operation_response<T: JsonSchema>(
  ctx: &mut aide::gen::GenContext,
  _operation: &mut aide::openapi::Operation,
  example: Option<serde_json::Value>,
  description: String,
) -> aide::openapi::Response {
  let schema = ctx.schema.subschema_for::<T>().into_object();

  aide::openapi::Response {
    // description: schema.metadata().description.clone().unwrap_or_default(),
    description,
    content: indexmap::IndexMap::from_iter([(
      "application/json".into(),
      aide::openapi::MediaType {
        schema: Some(aide::openapi::SchemaObject {
          json_schema: schema.into(),
          example: None,
          external_docs: None,
        }),
        example,
        ..Default::default()
      },
    )]),
    ..Default::default()
  }
}
