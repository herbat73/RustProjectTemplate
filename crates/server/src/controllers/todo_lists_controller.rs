use application::AppError;
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    routing::{get, put},
};
use types::{
    auth::AuthErrorBody,
    todo::{CreateTodoListCommand, TodosVm, UpdateTodoListCommand},
};

use crate::{error::ApiError, extractors::AccessUser, mapping::todo_list_dto, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_todo_lists).post(create_todo_list))
        .route("/{id}", put(update_todo_list).delete(delete_todo_list))
}

/// Retrieves all todo lists of the authenticated user along with their items.
#[utoipa::path(
    get,
    path = "/api/TodoLists",
    tag = "TodoLists",
    operation_id = "GetTodoLists",
    security(("access_token" = [])),
    responses(
        (status = 200, description = "OK", body = TodosVm),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn get_todo_lists(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
) -> Result<(StatusCode, Json<TodosVm>), ApiError> {
    let lists = state.todos.lists(&claims.user_uuid).await?;
    Ok((
        StatusCode::OK,
        Json(TodosVm::new(lists.into_iter().map(todo_list_dto).collect())),
    ))
}

/// Creates a new todo list and returns the ID of the created list.
#[utoipa::path(
    post,
    path = "/api/TodoLists",
    tag = "TodoLists",
    operation_id = "CreateTodoList",
    security(("access_token" = [])),
    request_body = CreateTodoListCommand,
    responses(
        (status = 201, description = "Created", body = i32),
        (status = 400, description = "Missing title or unsupported colour", body = AuthErrorBody),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn create_todo_list(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
    Json(command): Json<CreateTodoListCommand>,
) -> Result<(StatusCode, Json<i32>), ApiError> {
    let id = state
        .todos
        .create_list(&claims.user_uuid, command.title, command.colour)
        .await?;
    Ok((StatusCode::CREATED, Json(id)))
}

/// Updates the specified todo list. The ID in the URL must match the ID in the payload.
#[utoipa::path(
    put,
    path = "/api/TodoLists/{id}",
    tag = "TodoLists",
    operation_id = "UpdateTodoList",
    security(("access_token" = [])),
    params(("id" = i32, Path, description = "Todo list ID")),
    request_body = UpdateTodoListCommand,
    responses(
        (status = 204, description = "No Content"),
        (status = 400, description = "ID mismatch, missing title or unsupported colour", body = AuthErrorBody),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
        (status = 404, description = "Todo list not found (or owned by another user)", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn update_todo_list(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
    Path(id): Path<i32>,
    Json(command): Json<UpdateTodoListCommand>,
) -> Result<StatusCode, ApiError> {
    if id != command.id {
        return Err(AppError::BadRequest.into());
    }
    state
        .todos
        .update_list(&claims.user_uuid, id, command.title, command.colour)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes the todo list with the specified ID, together with its items.
#[utoipa::path(
    delete,
    path = "/api/TodoLists/{id}",
    tag = "TodoLists",
    operation_id = "DeleteTodoList",
    security(("access_token" = [])),
    params(("id" = i32, Path, description = "Todo list ID")),
    responses(
        (status = 204, description = "No Content"),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
        (status = 404, description = "Todo list not found (or owned by another user)", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn delete_todo_list(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    state.todos.delete_list(&claims.user_uuid, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
