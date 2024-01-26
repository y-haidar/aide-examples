use aide::{
  axum::{
    routing::{get_with, post_with, put_with},
    ApiRouter, IntoApiResponse,
  },
  transform::TransformOperation,
};
use axum::extract::State;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
  extractors::{self, code, EmptyResponse, JsonValidate, PathValidate},
  state::AppState,
};

use super::TodoItem;

pub fn todo_routes() -> ApiRouter<AppState> {
  ApiRouter::new()
    .api_route(
      "/",
      post_with(create_todo, create_todo_docs).get_with(list_todos, list_todos_docs),
    )
    .api_route(
      "/:id",
      get_with(get_todo, get_todo_docs).delete_with(delete_todo, delete_todo_docs),
    )
    .api_route("/:id/complete", put_with(complete_todo, complete_todo_docs))
  // .with_state(state)
}

/// New Todo details.
#[derive(Deserialize, JsonSchema, Validate)]
struct NewTodo {
  /// The description for the new Todo.
  description: String,
}

/// New Todo details.
#[derive(Serialize, JsonSchema)]
struct TodoCreated {
  /// The ID of the new Todo.
  id: Uuid,
}

async fn create_todo(
  State(app): State<AppState>,
  JsonValidate(todo): JsonValidate<NewTodo>,
) -> impl IntoApiResponse {
  let id = Uuid::new_v4();
  app.todos.lock().unwrap().insert(
    id,
    TodoItem {
      complete: false,
      description: todo.description,
      id,
    },
  );

  extractors::JsonOutput::<{ code(201) }, _>(TodoCreated { id })
}

fn create_todo_docs(op: TransformOperation) -> TransformOperation {
  op.description("Create a new incomplete Todo item.")
  // .response::<201, Json<TodoCreated>>()
}

#[derive(Serialize, JsonSchema)]
struct TodoList {
  todo_ids: Vec<Uuid>,
}

async fn list_todos(State(app): State<AppState>) -> axum::Json<TodoList> {
  axum::Json(TodoList {
    todo_ids: app.todos.lock().unwrap().keys().copied().collect(),
  })
}

fn list_todos_docs(op: TransformOperation) -> TransformOperation {
  op.description("List all Todo items.")
}

#[derive(Deserialize, JsonSchema, Validate)]
struct SelectTodo {
  /// The ID of the Todo.
  id: Uuid,
}

async fn get_todo(
  State(app): State<AppState>,
  PathValidate(todo): PathValidate<SelectTodo>,
) -> Result<axum::Json<TodoItem>, EmptyResponse<{ code(404) }>> {
  if let Some(todo) = app.todos.lock().unwrap().get(&todo.id) {
    Ok(axum::Json(todo.clone()))
  } else {
    Err(EmptyResponse())
  }
}

fn get_todo_docs(op: TransformOperation) -> TransformOperation {
  op.description("Get a single Todo item.")
  // .response_with::<200, Json<TodoItem>, _>(|res| {
  //     res.example(TodoItem {
  //         complete: false,
  //         description: "fix bugs".into(),
  //         id: Uuid::nil(),
  //     })
  // })
  // .response_with::<404, (), _>(|res| res.description("todo was not found"))
}

// Can quickly modify Response in fn signature
async fn delete_todo(
  State(app): State<AppState>,
  PathValidate(todo): PathValidate<SelectTodo>,
) -> Result<EmptyResponse<204>, EmptyResponse<404>> {
  if app.todos.lock().unwrap().remove(&todo.id).is_some() {
    // StatusCode::NO_CONTENT
    Ok(EmptyResponse())
  } else {
    // StatusCode::NOT_FOUND
    Err(EmptyResponse())
  }
}

fn delete_todo_docs(op: TransformOperation) -> TransformOperation {
  op.description("Delete a Todo item.")
    // You can still add description to responses, but make sure to match the status codes
    // Also if you do this `aide::gen::on_error` will fire, but you can ignore it
    .response_with::<204, (), _>(|res| res.description("The Todo has been deleted."))
    .response_with::<404, (), _>(|res| res.description("The todo was not found"))
}

async fn complete_todo(
  State(app): State<AppState>,
  PathValidate(todo): PathValidate<SelectTodo>,
) -> Result<EmptyResponse<204>, EmptyResponse<404>> {
  if let Some(todo) = app.todos.lock().unwrap().get_mut(&todo.id) {
    todo.complete = true;
    Ok(EmptyResponse())
  } else {
    Err(EmptyResponse())
  }
}

fn complete_todo_docs(op: TransformOperation) -> TransformOperation {
  op.description("Complete a Todo.")
  // .response::<204, ()>()
  // .response::<404, ()>()
}
