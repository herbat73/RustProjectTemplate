use application::AppError;
use axum::{
    RequestExt, Router,
    extract::{Json, Request, State},
    http::StatusCode,
    routing::{delete, get},
};
use types::{auth::AuthErrorBody, user::UserInfo};

use crate::{error::ApiError, extractors::AccessUser, mapping::user_info, state::AppState};

// route function to nest endpoints in router
pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/info", Router::new().route("/", get(get_user_info)))
        .nest("/all", Router::new().route("/", get(get_all_user_info)))
        .merge(Router::new().route("/", delete(delete_user)))
}

// get user info by JWT claims
/// Returns the authenticated user's info.
#[utoipa::path(
    get,
    path = "/user/info",
    tag = "user",
    security(("access_token" = [])),
    responses(
        (status = 200, description = "Authenticated user", body = UserInfo),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
        (status = 404, description = "User does not exist", body = AuthErrorBody),
    )
)]
pub async fn get_user_info(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
) -> Result<(StatusCode, Json<UserInfo>), ApiError> {
    let user = state.users.get(&claims.user_uuid).await?;
    Ok((StatusCode::OK, Json(user_info(user))))
}

/// Lists all users. Admin only.
#[utoipa::path(
    get,
    path = "/user/all",
    tag = "user",
    security(("access_token" = [])),
    responses(
        (status = 200, description = "All users", body = Vec<UserInfo>),
        (status = 403, description = "Missing or invalid token, or caller is not an admin", body = AuthErrorBody),
    )
)]
pub async fn get_all_user_info(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
) -> Result<(StatusCode, Json<Vec<UserInfo>>), ApiError> {
    let users = state.users.list_all(&claims).await?;
    Ok((
        StatusCode::OK,
        Json(users.into_iter().map(user_info).collect()),
    ))
}

/// Deletes a user by UUID. Admin only. The request body is the plain-text UUID.
#[utoipa::path(
    delete,
    path = "/user",
    tag = "user",
    security(("access_token" = [])),
    request_body(content = String, content_type = "text/plain", description = "UUID of the user to delete"),
    responses(
        (status = 200, description = "User deleted"),
        (status = 403, description = "Missing or invalid token, or caller is not an admin", body = AuthErrorBody),
        (status = 404, description = "User does not exist", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn delete_user(
    State(state): State<AppState>,
    AccessUser(claims): AccessUser,
    request: Request,
) -> Result<StatusCode, ApiError> {
    let uuid: String = request.extract().await.map_err(|error| {
        AppError::Internal(format!(
            "Could not read the user uuid from the request body: {error}"
        ))
    })?;
    state.users.delete(&claims, &uuid).await?;
    Ok(StatusCode::OK)
}
