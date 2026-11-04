use axum::{
    Router,
    extract::{Json, Path},
    http::{HeaderMap, StatusCode},
    middleware,
    routing::{get, put},
};
use types::{
    auth::{AuthErrorBody, AuthErrorType},
    todo::{COLOURS, CreateTodoListCommand, TodosVm, UpdateTodoListCommand},
};

use crate::{
    middleware::token_authentication,
    strategies::{
        authentication::{AuthClaims, AuthError, Claims},
        todo,
        users::get_db_user_by_uuid,
    },
};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(get_todo_lists).post(create_todo_list))
        .route("/{id}", put(update_todo_list).delete(delete_todo_list))
        .layer(middleware::from_fn(
            token_authentication::authenticate_token::<AuthClaims>,
        ))
}

/// Resolves the id of the authenticated user from the verified token claims.
pub(super) async fn current_user_id(headers: &HeaderMap) -> Result<i32, AuthError> {
    let claims = AuthClaims::from_header(headers);
    get_db_user_by_uuid(claims.sub)
        .await
        .map(|user| user.id)
        .map_err(|_| AuthError::from_error_type(AuthErrorType::UserDoesNotExist))
}

fn db_error(error: sqlx::Error) -> AuthError {
    println!("{error}");
    AuthError::from_error_type(AuthErrorType::ServerError)
}

/// A title is required and limited to 200 characters.
pub(super) fn validate_title(title: Option<String>) -> Result<String, AuthError> {
    match title.map(|t| t.trim().to_string()) {
        Some(t) if !t.is_empty() && t.chars().count() <= 200 => Ok(t),
        _ => Err(AuthError::from_error_type(AuthErrorType::MissingFields)),
    }
}

/// Normalizes a colour code (case-insensitive) to one of the supported codes.
fn validate_colour(colour: Option<String>) -> Result<Option<String>, AuthError> {
    match colour {
        None => Ok(None),
        Some(c) => COLOURS
            .iter()
            .find(|(code, _)| code.eq_ignore_ascii_case(c.trim()))
            .map(|(code, _)| Some((*code).to_string()))
            .ok_or_else(|| AuthError::from_error_type(AuthErrorType::BadRequest)),
    }
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
pub async fn get_todo_lists(headers: HeaderMap) -> Result<(StatusCode, Json<TodosVm>), AuthError> {
    let lists = todo::get_lists_with_items(current_user_id(&headers).await?)
        .await
        .map_err(db_error)?;
    Ok((StatusCode::OK, Json(TodosVm::new(lists))))
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
    headers: HeaderMap,
    Json(command): Json<CreateTodoListCommand>,
) -> Result<(StatusCode, Json<i32>), AuthError> {
    let user_id = current_user_id(&headers).await?;
    let title = validate_title(command.title)?;
    let colour = validate_colour(command.colour)?.unwrap_or_else(|| COLOURS[0].0.to_string());
    let id = todo::insert_list(user_id, title, colour)
        .await
        .map_err(db_error)?;
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
    headers: HeaderMap,
    Path(id): Path<i32>,
    Json(command): Json<UpdateTodoListCommand>,
) -> Result<StatusCode, AuthError> {
    if id != command.id {
        return Err(AuthError::from_error_type(AuthErrorType::BadRequest));
    }
    let title = validate_title(command.title)?;
    let colour = validate_colour(command.colour)?;
    let user_id = current_user_id(&headers).await?;
    if todo::update_list(user_id, id, title, colour)
        .await
        .map_err(db_error)?
    {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AuthError::from_error_type(AuthErrorType::NotFound))
    }
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
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, AuthError> {
    let user_id = current_user_id(&headers).await?;
    if todo::delete_list(user_id, id).await.map_err(db_error)? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AuthError::from_error_type(AuthErrorType::NotFound))
    }
}
