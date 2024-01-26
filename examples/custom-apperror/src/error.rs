use axum::{extract::rejection::PathRejection, http::StatusCode, response::IntoResponse};
// use axum_jsonschema::JsonSchemaRejection;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;
use validator::ValidationErrors;

/// A default error response for most API errors.
#[derive(Debug, Serialize, JsonSchema)]
// #[derive(aide::OperationIo)]
// #[aide(output_with = "axum::Json<AppError>", json_schema)]
pub struct AppError {
    /// An error message.
    pub error: String,
    /// A unique error ID.
    pub error_id: Uuid,
    #[serde(skip)]
    pub status: StatusCode,
    /// Optional Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_details: Option<Value>,
}

impl AppError {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
            error_id: Uuid::new_v4(),
            status: StatusCode::BAD_REQUEST,
            error_details: None,
        }
    }

    // pub fn with_status(mut self, status: StatusCode) -> Self {
    //     self.status = status;
    //     self
    // }

    // pub fn with_details(mut self, details: Value) -> Self {
    //     self.error_details = Some(details);
    //     self
    // }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status;
        let mut res = axum::Json(self).into_response();
        *res.status_mut() = status;
        res
    }
}

// impl From<RawPathParamsRejection> for AppError {
//   fn from(rejection: RawPathParamsRejection) -> Self {
//     match rejection {
//       RawPathParamsRejection::InvalidUtf8InPathParam(e) => Self::new("invalid request")
//         .with_details(serde_json::json!({"path_invalid_utf8": &format!("{e:#?}")})),
//       RawPathParamsRejection::MissingPathParams(e) => Self::new("invalid request")
//         .with_details(serde_json::json!({"path_missing": &format!("{e:#?}")})),
//       _ => Self::new("invalid request")
//         .with_details(serde_json::json!({"path_other": &format!("{rejection:#?}")})),
//     }
//   }
// }

// impl From<JsonRejection> for AppError {
//   fn from(rejection: JsonRejection) -> Self {
//     AppError::new(&serde_json::json!({ "error": rejection.to_string() }).to_string())
//   }
// }

impl From<ValidationErrors> for AppError {
    fn from(value: ValidationErrors) -> Self {
        AppError::new(&value.to_string())
    }
}

impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        AppError::new(&serde_json::json!({ "error": rejection.to_string() }).to_string())
    }
}

// impl From<JsonSchemaRejection> for AppError {
//     fn from(rejection: JsonSchemaRejection) -> Self {
//         match rejection {
//             JsonSchemaRejection::Json(j) => Self::new(&j.to_string()),
//             JsonSchemaRejection::Serde(_) => Self::new("invalid request"),
//             JsonSchemaRejection::Schema(s) => Self::new("invalid request")
//                 .with_details(serde_json::json!({ "schema_validation": s })),
//         }
//     }
// }

impl aide::OperationOutput for AppError {
    type Inner = Self;

    fn operation_response(
        ctx: &mut aide::gen::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        <axum::Json<AppError> as aide::OperationOutput>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut aide::gen::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        if let Some(res) = Self::operation_response(ctx, operation) {
            vec![(None, res)]
        } else {
            Vec::new()
        }
        // vec![(
        //   Some(StatusCode::ACCEPTED.into()),
        //   Response {
        //     description: "An Error response".to_owned(),
        //     // headers: todo!(),
        //     // content: todo!(),
        //     ..Default::default()
        //   },
        // )]
    }
}
