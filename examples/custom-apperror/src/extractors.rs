use crate::error::{AppError, AppErrorCode, AppErrorOutput};
use axum::{
  extract::{FromRequest, FromRequestParts, Request},
  http::request::Parts,
  response::IntoResponse,
};
// use axum_macros::FromRequest;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use validator::Validate;

// #[derive(OperationIo)]
// // #[from_request(via(axum::Json), rejection(AppError))]
// #[aide(
//   input_with = "axum::Json<T>",
// //   output_with = "axum::Json<T>",
//   json_schema
// )]
pub struct JsonValidate<T>(pub T);

// To add validation in extractors
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
    // println!("hmmm json_val input");
    [AppErrorCode::Validation, AppErrorCode::JsonRejection]
      .into_iter()
      .for_each(|status| set_inferred_response(ctx, operation, status));

    // <axum::Json<AppErrorOutput> as aide::OperationInput>::operation_input(ctx, operation);
    <axum::Json<T> as aide::OperationInput>::operation_input(ctx, operation)
  }
}

// =======

pub struct JsonOutput<const N: u16, T>(pub T);

impl<const N: u16, T> IntoResponse for JsonOutput<N, T>
where
  T: Serialize,
{
  fn into_response(self) -> axum::response::Response {
    unsafe {
      (
        axum::http::StatusCode::from_u16(N).unwrap_unchecked(),
        axum::Json(self.0),
      )
        .into_response()
    }
  }
}

pub const fn code(n: u16) -> u16 {
  if n < 100 || n >= 1000 {
    panic!("Err(InvalidStatusCode::new())");
  }
  n
}

impl<const N: u16, T: JsonSchema> aide::OperationOutput for JsonOutput<N, T> {
  type Inner = T;

  fn operation_response(
    ctx: &mut aide::gen::GenContext,
    operation: &mut aide::openapi::Operation,
  ) -> Option<aide::openapi::Response> {
    // println!("hmmm JsonOutput oper");
    <axum::Json<T> as aide::OperationOutput>::operation_response(ctx, operation)
  }

  fn inferred_responses(
    ctx: &mut aide::gen::GenContext,
    operation: &mut aide::openapi::Operation,
  ) -> Vec<(Option<u16>, aide::openapi::Response)> {
    // println!("hmmm JsonOutput infer");
    if let Some(res) = Self::operation_response(ctx, operation) {
      vec![(Some(N), res)]
    } else {
      Vec::new()
    }
  }
}

// ==============

pub struct EmptyResponse<const N: u16>();

impl<const N: u16> IntoResponse for EmptyResponse<N> {
  fn into_response(self) -> axum::response::Response {
    unsafe { (axum::http::StatusCode::from_u16(N).unwrap_unchecked(),).into_response() }
  }
}

impl<const N: u16> aide::OperationOutput for EmptyResponse<N> {
  type Inner = ();

  fn inferred_responses(
    _ctx: &mut aide::gen::GenContext,
    _operation: &mut aide::openapi::Operation,
  ) -> Vec<(Option<u16>, aide::openapi::Response)> {
    let res = aide::openapi::Response::default();
    vec![(Some(N), res)]
  }
}

// ==============

// #[derive(OperationIo)]
// #[aide(
//   input_with = "axum::extract::Path<T>",
//   // output_with = "axum_jsonschema::Json<T>",
//   json_schema
// )]
pub struct PathValidate<T>(pub T);

// To add validation in extractors
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
    // println!("hmmm path_val input");
    // let res = unsafe {
    //   <axum::Json<AppErrorOutput> as aide::OperationOutput>::operation_response(ctx, operation)
    //     .unwrap_unchecked()
    // };
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

fn set_inferred_response(
  ctx: &mut aide::gen::GenContext,
  operation: &mut aide::openapi::Operation,
  status: AppErrorCode,
) {
  use aide::openapi::{ReferenceOr, StatusCode};

  let example = unsafe { serde_json::to_value(status.create_example()).unwrap_unchecked() };
  let description = status.create_description();
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
