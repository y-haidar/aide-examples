use std::sync::Arc;

use aide::{
  axum::{
    routing::{get, get_with},
    ApiRouter, IntoApiResponse,
  },
  openapi::OpenApi,
  redoc::Redoc,
  scalar::Scalar,
};
use axum::{response::IntoResponse, Extension, Json};

pub fn docs_routes() -> ApiRouter {
  let router = ApiRouter::new()
    .api_route_with(
      "/",
      get_with(
        Scalar::new("/docs/private/api.json")
          .with_title("Aide Axum")
          .axum_handler(),
        |op| op.description("This documentation page."),
      ),
      |p| p.security_requirement("ApiKey"),
    )
    .api_route_with(
      "/redoc",
      get_with(
        Redoc::new("/docs/private/api.json")
          .with_title("Aide Axum")
          .axum_handler(),
        |op| op.description("This documentation page."),
      ),
      |p| p.security_requirement("ApiKey"),
    )
    .route("/private/api.json", get(serve_docs));

  router
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
  // `into_response` Turns Json<T> into `Response`, thus
  // no `Component` will be generated from this route
  Json(api).into_response()
}
