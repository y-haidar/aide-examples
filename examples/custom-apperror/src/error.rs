use axum::{
  extract::rejection::{JsonRejection, PathRejection},
  http::StatusCode,
  response::IntoResponse,
};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{json, Value};
use strum::IntoEnumIterator;
use strum_macros::{EnumDiscriminants, EnumIter};
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

// 415 unsupported media type
// 413 content too large

/// Error response for most API errors.
#[derive(thiserror::Error, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(AppErrorCode))]
#[allow(dead_code)]
pub enum AppError {
  #[error("{0}")]
  Internal(&'static str),
  #[error(transparent)]
  Validation(#[from] ValidationErrors),
  #[error(transparent)]
  PathRejection(#[from] PathRejection),
  #[error(transparent)]
  JsonRejection(#[from] JsonRejection),
  #[error("API Key was not provided")]
  AuthKeyMissing,
  #[error("API Key is not invalid")]
  AuthKeyInvalid,
  #[error("API Key does not have access")]
  AuthKeyNoAccess,
}

impl AppError {
  fn to_app_error_output(&self) -> AppErrorOutput {
    match self {
      AppError::Internal(error) => AppErrorOutput::new(error, None),
      AppError::Validation(e) => {
        // you can match on err here, but for sake of keeping it short not going to
        AppErrorOutput::new(
          "Validation Failed",
          Some(unsafe { serde_json::to_value(e).unwrap_unchecked() }),
        )
      }
      AppError::PathRejection(e) => {
        AppErrorOutput::new("Incorrect Path", Some(json!(e.to_string())))
      }
      AppError::JsonRejection(e) => {
        AppErrorOutput::new("Incorrect Json", Some(json!(e.to_string())))
      }
      AppError::AuthKeyMissing | AppError::AuthKeyInvalid | AppError::AuthKeyNoAccess => {
        AppErrorOutput::new("Unauthorized", None)
      }
    }
  }
}

impl AppErrorCode {
  pub fn status_code(&self) -> StatusCode {
    match self {
      AppErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
      AppErrorCode::Validation => StatusCode::UNPROCESSABLE_ENTITY,
      AppErrorCode::PathRejection => StatusCode::BAD_REQUEST,
      AppErrorCode::JsonRejection => StatusCode::BAD_REQUEST,
      AppErrorCode::AuthKeyMissing | AppErrorCode::AuthKeyInvalid => StatusCode::UNAUTHORIZED,
      AppErrorCode::AuthKeyNoAccess => StatusCode::FORBIDDEN,
    }
  }

  pub fn status_code_as_u16(&self) -> u16 {
    self.status_code().as_u16()
  }

  pub fn create_example(&self) -> AppErrorOutput {
    match self {
      AppErrorCode::Internal => AppErrorOutput::new("Internal", None),
      AppErrorCode::Validation => {
        #[derive(Validate)]
        struct Example {
          #[validate(length(min = 5))]
          param1: &'static str,
          #[validate(range(min = 18, max = 20))]
          param2: u32,
        }
        let ex = unsafe {
          Example {
            param1: "test",
            param2: 10,
          }
          .validate()
          .unwrap_err_unchecked()
        };
        AppError::Validation(ex).to_app_error_output()
      }
      AppErrorCode::PathRejection => AppErrorOutput::new("Incorrect Path", None),
      AppErrorCode::JsonRejection => AppErrorOutput::new("Incorrect Json", None),
      AppErrorCode::AuthKeyMissing
      | AppErrorCode::AuthKeyInvalid
      | AppErrorCode::AuthKeyNoAccess => AppErrorOutput::new("Unauthorized", None),
    }
  }

  pub fn description(&self) -> &'static str {
    match self {
      AppErrorCode::Internal => "A generic internal error",
      AppErrorCode::Validation => "A validation error",
      AppErrorCode::PathRejection => "The path parameters were not supplied correctly",
      AppErrorCode::JsonRejection => "A json deserialization error",
      AppErrorCode::AuthKeyInvalid
      | AppErrorCode::AuthKeyMissing
      | AppErrorCode::AuthKeyNoAccess => {
        "The API Key was not provided, incorrect, or does not have access rights"
      }
    }
  }
}

impl IntoResponse for AppError {
  fn into_response(self) -> axum::response::Response {
    let json_output = axum::Json(self.to_app_error_output());

    let code: AppErrorCode = self.into();
    let code = code.status_code();
    (code, json_output).into_response()
  }
}

/// Error response for most API errors.
#[derive(Debug, Serialize, JsonSchema)]
#[schemars(example = "app_error_output_examples")]
#[serde(rename = "AppError")]
pub struct AppErrorOutput {
  /// An error message.
  pub error: String,
  /// A unique error ID.
  pub error_id: Uuid,
  /// Optional Additional error details.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_details: Option<Value>,
}

fn app_error_output_examples() -> Vec<AppErrorOutput> {
  AppErrorCode::iter()
    .map(|code| code.create_example())
    .collect()
}

impl AppErrorOutput {
  pub fn new(error: &str, error_details: Option<Value>) -> Self {
    Self {
      error: error.to_string(),
      error_id: Uuid::new_v4(),
      // status: StatusCode::BAD_REQUEST,
      error_details,
    }
  }
}

// Only needed if used as output of routes, impl might be wrong tho
// impl aide::OperationOutput for AppError {
//   type Inner = ();
//   fn operation_response(
//     ctx: &mut aide::gen::GenContext,
//     operation: &mut aide::openapi::Operation,
//   ) -> Option<aide::openapi::Response> {
//     <axum::Json<AppErrorOutput> as aide::OperationOutput>::operation_response(ctx, operation)
//   }
//   fn inferred_responses(
//     ctx: &mut aide::gen::GenContext,
//     operation: &mut aide::openapi::Operation,
//   ) -> Vec<(Option<u16>, aide::openapi::Response)> {
//     // println!("hmmm");
//     if let Some(res) = Self::operation_response(ctx, operation) {
//       AppErrorCode::iter()
//         .map(|c| (Some(c.status_code_as_u16()), res.clone()))
//         .collect()
//     } else {
//       Vec::new()
//     }
//     // vec![(
//     //   Some(StatusCode::ACCEPTED.into()),
//     //   Response {
//     //     description: "An Error response".to_owned(),
//     //     // headers: todo!(),
//     //     // content: todo!(),
//     //     ..Default::default()
//     //   },
//     // )]
//   }
// }
