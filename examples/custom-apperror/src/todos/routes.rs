use aide::{
  axum::{
    routing::{get_with, post_with, put_with},
    ApiRouter, IntoApiResponse,
  },
  transform::TransformOperation,
};
use axum::{extract::State, http::StatusCode, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
  extractors::{auth::ApiKey, JsonValidate, PathValidate},
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
  _: ApiKey,
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

  Json(TodoCreated { id })
}

fn create_todo_docs(op: TransformOperation) -> TransformOperation {
  op.description("Create a new incomplete Todo item.")
    .response::<201, Json<TodoCreated>>()
}

#[derive(Serialize, JsonSchema)]
struct TodoList {
  todo_ids: Vec<Uuid>,
}

async fn list_todos(State(app): State<AppState>) -> Json<TodoList> {
  Json(TodoList {
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
) -> Result<Json<TodoItem>, StatusCode> {
  if let Some(todo) = app.todos.lock().unwrap().get(&todo.id) {
    Ok(Json(todo.clone()))
  } else {
    Err(StatusCode::NOT_FOUND)
  }
}

fn get_todo_docs(op: TransformOperation) -> TransformOperation {
  op.description("Get a single Todo item.")
    .response_with::<200, Json<TodoItem>, _>(|res| {
      res.example(TodoItem {
        complete: false,
        description: "fix bugs".into(),
        id: Uuid::nil(),
      })
    })
    .response_with::<404, (), _>(|res| res.description("todo was not found"))
}

// Can quickly modify Response in fn signature
async fn delete_todo(
  State(app): State<AppState>,
  PathValidate(todo): PathValidate<SelectTodo>,
) -> StatusCode {
  if app.todos.lock().unwrap().remove(&todo.id).is_some() {
    StatusCode::NO_CONTENT
  } else {
    StatusCode::NOT_FOUND
  }
}

fn delete_todo_docs(op: TransformOperation) -> TransformOperation {
  op.description("Delete a Todo item.")
    .response_with::<204, (), _>(|res| res.description("The Todo has been deleted."))
    .response_with::<404, (), _>(|res| res.description("The todo was not found"))
}

async fn complete_todo(
  State(app): State<AppState>,
  PathValidate(todo): PathValidate<SelectTodo>,
) -> StatusCode {
  if let Some(todo) = app.todos.lock().unwrap().get_mut(&todo.id) {
    todo.complete = true;
    StatusCode::NO_CONTENT
  } else {
    StatusCode::NOT_FOUND
  }
}

fn complete_todo_docs(op: TransformOperation) -> TransformOperation {
  op.description("Complete a Todo.")
    .response::<204, ()>()
    .response::<404, ()>()
}
