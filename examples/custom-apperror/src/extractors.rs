use crate::error::AppError;
use aide::OperationIo;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};
// use axum_macros::FromRequest;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};
use validator::Validate;

// #[derive(FromRequest, OperationIo)]
// #[from_request(via(axum::Json), rejection(AppError))]
// #[aide(
//   input_with = "axum::Json<T>",
//   // output_with = "axum::Json<T>",
//   json_schema
// )]
pub struct Json<const N: u16, T>(pub T);

impl<const N: u16, T> IntoResponse for Json<N, T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        unsafe {
            (
                StatusCode::from_u16(N).unwrap_unchecked(),
                axum::Json(self.0),
            )
                .into_response()
        }
    }
}
// pub fn from_u16(src: u16) -> Result<StatusCode, InvalidStatusCode> {
//     if src < 100 || src >= 1000 {
//         return Err(InvalidStatusCode::new());
//     }

//     NonZeroU16::new(src)
//         .map(StatusCode)
//         .ok_or_else(InvalidStatusCode::new)
// }

pub const fn code(n: u16) -> u16 {
    if n < 100 || n >= 1000 {
        panic!("Err(InvalidStatusCode::new())");
    }
    n
}

impl<const N: u16, T: JsonSchema> aide::OperationOutput for Json<N, T> {
    type Inner = T;

    fn operation_response(
        ctx: &mut aide::gen::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        <axum::Json<T> as aide::OperationOutput>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut aide::gen::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        if let Some(res) = Self::operation_response(ctx, operation) {
            vec![(Some(N), res)]
        } else {
            Vec::new()
        }
    }
}

// ==============

#[derive(OperationIo)]
#[aide(
  input_with = "axum::extract::Path<T>",
  // output_with = "axum_jsonschema::Json<T>",
  json_schema
)]
pub struct Path<T>(pub T);

// To add validation in extractors
#[axum::async_trait]
impl<T, S> FromRequestParts<S> for Path<T>
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
        Ok(Path(value))
    }
}
