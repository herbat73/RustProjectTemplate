use axum::{
    Router,
    extract::{Json, Path},
    http::{HeaderMap, StatusCode},
    middleware,
    routing::{patch, post, put},
};
use types::{
    auth::{AuthErrorBody, AuthErrorType},
    todo::{
        CreateTodoItemCommand, PRIORITY_LEVELS, UpdateTodoItemCommand, UpdateTodoItemDetailCommand,
    },
};

use super::todo_lists_controller::{current_user_id, validate_title};
use crate::{
    middleware::token_authentication,
    strategies::{
        authentication::{AuthClaims, AuthError},
        todo,
    },
};

pub fn routes() -> Router {
    Router::new()
        .route("/", post(create_todo_item))
        .route("/{id}", put(update_todo_item).delete(delete_todo_item))
        .route("/UpdateDetail/{id}", patch(update_todo_item_detail))
        .layer(middleware::from_fn(
            token_authentication::authenticate_token::<AuthClaims>,
        ))
}

fn db_error(error: sqlx::Error) -> AuthError {
    println!("{error}");
    AuthError::from_error_type(AuthErrorType::ServerError)
}

fn not_found() -> AuthError {
    AuthError::from_error_type(AuthErrorType::NotFound)
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
    headers: HeaderMap,
    Json(command): Json<CreateTodoItemCommand>,
) -> Result<(StatusCode, Json<i32>), AuthError> {
    let user_id = current_user_id(&headers).await?;
    let title = validate_title(command.title)?;
    if !todo::list_exists(user_id, command.list_id)
        .await
        .map_err(db_error)?
    {
        return Err(not_found());
    }
    let id = todo::insert_item(command.list_id, title)
        .await
        .map_err(db_error)?;
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
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(command): Json<UpdateTodoItemCommand>,
) -> Result<StatusCode, AuthError> {
    if id != command.id {
        return Err(AuthError::from_error_type(AuthErrorType::BadRequest));
    }
    let title = validate_title(command.title)?;
    let user_id = current_user_id(&headers).await?;
    if todo::update_item(user_id, id, title, command.done)
        .await
        .map_err(db_error)?
    {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found())
    }
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
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(command): Json<UpdateTodoItemDetailCommand>,
) -> Result<StatusCode, AuthError> {
    if id != command.id
        || !PRIORITY_LEVELS
            .iter()
            .any(|(level, _)| *level == command.priority)
    {
        return Err(AuthError::from_error_type(AuthErrorType::BadRequest));
    }
    let user_id = current_user_id(&headers).await?;
    if !todo::list_exists(user_id, command.list_id)
        .await
        .map_err(db_error)?
    {
        return Err(not_found());
    }
    if todo::update_item_detail(user_id, id, command.list_id, command.priority, command.note)
        .await
        .map_err(db_error)?
    {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found())
    }
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
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, AuthError> {
    let user_id = current_user_id(&headers).await?;
    if todo::delete_item(user_id, id).await.map_err(db_error)? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found())
    }
}
