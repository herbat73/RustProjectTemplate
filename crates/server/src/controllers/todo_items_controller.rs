use application::AppError;
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    routing::{patch, post, put},
};
use types::{
    auth::AuthErrorBody,
    todo::{CreateTodoItemCommand, UpdateTodoItemCommand, UpdateTodoItemDetailCommand},
};

use crate::{error::ApiError, extractors::AccessUser, state::AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_todo_item))
        .route("/{id}", put(update_todo_item).delete(delete_todo_item))
        .route("/UpdateDetail/{id}", patch(update_todo_item_detail))
}

/// Creates a new todo item and returns the ID of the created item.
#[utoipa::path(
    post,
    path = "/api/TodoItems",
    tag = "TodoItems",
    operation_id = "CreateTodoItem",
    security(("access_token" = [])),
    request_body = CreateTodoItemCommand,
    responses(
        (status = 201, description = "Created", body = i32),
        (status = 400, description = "Missing title", body = AuthErrorBody),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
        (status = 404, description = "Todo list not found (or owned by another user)", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn create_todo_item(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
    Json(command): Json<CreateTodoItemCommand>,
) -> Result<(StatusCode, Json<i32>), ApiError> {
    let id = state
        .todos
        .create_item(&claims.user_uuid, command.list_id, command.title)
        .await?;
    Ok((StatusCode::CREATED, Json(id)))
}

/// Updates the specified todo item. The ID in the URL must match the ID in the payload.
#[utoipa::path(
    put,
    path = "/api/TodoItems/{id}",
    tag = "TodoItems",
    operation_id = "UpdateTodoItem",
    security(("access_token" = [])),
    params(("id" = i32, Path, description = "Todo item ID")),
    request_body = UpdateTodoItemCommand,
    responses(
        (status = 204, description = "No Content"),
        (status = 400, description = "ID mismatch or missing title", body = AuthErrorBody),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
        (status = 404, description = "Todo item not found (or owned by another user)", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn update_todo_item(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
    Path(id): Path<i32>,
    Json(command): Json<UpdateTodoItemCommand>,
) -> Result<StatusCode, ApiError> {
    if id != command.id {
        return Err(AppError::BadRequest.into());
    }
    state
        .todos
        .update_item(&claims.user_uuid, id, command.title, command.done)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Updates the detail fields (list, priority, note) of a todo item. The ID in the URL must match the ID in the payload.
#[utoipa::path(
    patch,
    path = "/api/TodoItems/UpdateDetail/{id}",
    tag = "TodoItems",
    operation_id = "UpdateTodoItemDetail",
    security(("access_token" = [])),
    params(("id" = i32, Path, description = "Todo item ID")),
    request_body = UpdateTodoItemDetailCommand,
    responses(
        (status = 204, description = "No Content"),
        (status = 400, description = "ID mismatch or unsupported priority level", body = AuthErrorBody),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
        (status = 404, description = "Todo item or target list not found (or owned by another user)", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn update_todo_item_detail(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
    Path(id): Path<i32>,
    Json(command): Json<UpdateTodoItemDetailCommand>,
) -> Result<StatusCode, ApiError> {
    if id != command.id {
        return Err(AppError::BadRequest.into());
    }
    state
        .todos
        .update_item_detail(
            &claims.user_uuid,
            id,
            command.list_id,
            command.priority,
            command.note,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes the todo item with the specified ID.
#[utoipa::path(
    delete,
    path = "/api/TodoItems/{id}",
    tag = "TodoItems",
    operation_id = "DeleteTodoItem",
    security(("access_token" = [])),
    params(("id" = i32, Path, description = "Todo item ID")),
    responses(
        (status = 204, description = "No Content"),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
        (status = 404, description = "Todo item not found (or owned by another user)", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn delete_todo_item(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    state.todos.delete_item(&claims.user_uuid, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
