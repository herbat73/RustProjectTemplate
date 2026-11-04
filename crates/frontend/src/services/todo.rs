use reqwest::{RequestBuilder, Response};
use types::auth::AuthErrorType;
use types::todo::{CreateTodoItemCommand, CreateTodoListCommand, TodosVm, UpdateTodoItemCommand};

use super::{AuthError, AuthRequest, get_base_url, get_http_client};

async fn send(builder: RequestBuilder) -> Result<Response, AuthError> {
    let response = AuthRequest::new(builder).send().await?;
    if !response.status().is_success() {
        return Err(AuthError::from_response(response).await);
    }
    Ok(response)
}

/// Fetches all todo lists (with items) of the logged-in user.
pub async fn get_todos() -> Result<TodosVm, AuthError> {
    send(get_http_client().get(get_base_url() + "/api/TodoLists"))
        .await?
        .json::<TodosVm>()
        .await
        .map_err(|_| AuthError::from_error_type(AuthErrorType::BadRequest))
}

pub async fn create_list(title: String) -> Result<(), AuthError> {
    let command = CreateTodoListCommand {
        title: Some(title),
        colour: None,
    };
    send(
        get_http_client()
            .post(get_base_url() + "/api/TodoLists")
            .json(&command),
    )
    .await?;
    Ok(())
}

pub async fn delete_list(id: i32) -> Result<(), AuthError> {
    send(get_http_client().delete(format!("{}/api/TodoLists/{id}", get_base_url()))).await?;
    Ok(())
}

pub async fn create_item(list_id: i32, title: String) -> Result<(), AuthError> {
    let command = CreateTodoItemCommand {
        list_id,
        title: Some(title),
    };
    send(
        get_http_client()
            .post(get_base_url() + "/api/TodoItems")
            .json(&command),
    )
    .await?;
    Ok(())
}

pub async fn update_item(id: i32, title: String, done: bool) -> Result<(), AuthError> {
    let command = UpdateTodoItemCommand {
        id,
        title: Some(title),
        done,
    };
    send(
        get_http_client()
            .put(format!("{}/api/TodoItems/{id}", get_base_url()))
            .json(&command),
    )
    .await?;
    Ok(())
}

pub async fn delete_item(id: i32) -> Result<(), AuthError> {
    send(get_http_client().delete(format!("{}/api/TodoItems/{id}", get_base_url()))).await?;
    Ok(())
}
