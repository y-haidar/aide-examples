use std::sync::Arc;

use aide::{axum::ApiRouter, openapi::OpenApi, transform::TransformOpenApi};
use axum::Extension;
use docs::docs_routes;
use state::AppState;
use todos::routes::todo_routes;

mod docs;
mod error;
mod extractors;
mod state;
mod todos;

#[tokio::main]
async fn main() {
    aide::gen::on_error(|error| {
        println!("{error}");
    });
    aide::gen::extract_schemas(true);

    let state = AppState::default();
    let mut api = OpenApi::default();

    let app = ApiRouter::new()
        // .nest_api_service("/todo", todo_routes().with_state(state))
        .nest_api_service("/docs", docs_routes())
        .finish_api(&mut api)
        // .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api)));

    println!("Example docs are accessible at http://127.0.0.1:3001/docs");

    let tcp_listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();

    axum::serve(tcp_listener, app).await.unwrap();
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Aide axum Open API")
        .summary("An example Todo application")
        // .description(include_str!("README.md"))
        .description("My description")
        .tag(aide::openapi::Tag {
            name: "todo".into(),
            description: Some("Todo Management".into()),
            ..Default::default()
        })
        .security_scheme(
            "ApiKey",
            aide::openapi::SecurityScheme::ApiKey {
                location: aide::openapi::ApiKeyLocation::Header,
                name: "X-Auth-Key".into(),
                description: Some("A key that is ignored.".into()),
                extensions: Default::default(),
            },
        )
        .default_response_with::<axum::Json<error::AppError>, _>(|res| {
            res.example(error::AppError {
                error: "some error happened".to_string(),
                error_details: None,
                error_id: uuid::Uuid::nil(),
                // This is not visible.
                status: axum::http::StatusCode::IM_A_TEAPOT,
            })
        })
    // .summary("summary")
    // .description("description")
}
