use std::sync::Arc;

use aide::{openapi::OpenApi, redoc::Redoc, scalar::Scalar};
use axum::{response::IntoResponse, routing::get, Extension, Json, Router};

pub fn docs_routes() -> Router {
  let router = Router::new()
    .route(
      "/",
      get(
        Scalar::new("/docs/private/api.json")
          .with_title("Aide Axum")
          .axum_handler(),
      ),
    )
    .route(
      "/redoc",
      get(
        Redoc::new("/docs/private/api.json")
          .with_title("Aide Axum")
          .axum_handler(),
      ),
    )
    .route("/private/api.json", get(serve_docs));

  router
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoResponse {
  // `into_response` Turns Json<T> into `Response`, thus
  // no `Component` will be generated from this route
  Json(api).into_response()
}
