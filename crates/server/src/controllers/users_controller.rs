use axum::{
    RequestExt, Router,
    extract::{Json, Request},
    http::StatusCode,
    middleware,
    routing::{delete, get},
};

use types::{
    auth::{AuthErrorBody, AuthErrorType},
    user::UserInfo,
};

use crate::{
    middleware::token_authentication,
    strategies::{
        authentication::{AuthClaims, AuthError, Claims},
        users::{delete_user_by_uuid, get_all_users, get_db_user_by_uuid},
    },
};

// route function to nest endpoints in router
pub fn routes() -> Router {
    // create routes
    Router::new()
        .nest(
            "/info",
            Router::new()
                .route("/", get(get_user_info))
                .layer(middleware::from_fn(
                    token_authentication::authenticate_token::<AuthClaims>,
                )),
        )
        .nest(
            "/all",
            Router::new()
                .route("/", get(get_all_user_info))
                .layer(middleware::from_fn(
                    token_authentication::authenticate_token::<AuthClaims>,
                )),
        )
        .merge(
            Router::new()
                .route("/", delete(delete_user))
                .layer(middleware::from_fn(
                    token_authentication::authenticate_token::<AuthClaims>,
                )),
        )
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
pub async fn get_user_info(request: Request) -> Result<(StatusCode, Json<UserInfo>), AuthError> {
    // generate AuthClaims from encoded x-claim header
    let claims = AuthClaims::from_header(request.headers());
    match get_db_user_by_uuid(claims.sub).await {
        Ok(user) => Ok((StatusCode::OK, axum::Json(UserInfo::from_user(user)))),
        Err(_) => Err(AuthError::from_error_type(AuthErrorType::UserDoesNotExist)),
    }
}

// get user info by JWT claims
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
    request: Request,
) -> Result<(StatusCode, Json<Vec<UserInfo>>), AuthError> {
    // generate AuthRequesterClaims from encoded x-claim header
    let claims = AuthClaims::from_header(request.headers());
    if claims.acc {
        match get_all_users().await {
            Ok(users) => Ok((StatusCode::OK, axum::Json(users))),
            Err(_) => Err(AuthError::from_error_type(AuthErrorType::UserDoesNotExist)),
        }
    } else {
        Err(AuthError::from_error_type(AuthErrorType::AccessDenied))
    }
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
pub async fn delete_user(request: Request) -> Result<StatusCode, AuthError> {
    // generate AuthRequesterClaims from encoded x-claim header
    let claims = AuthClaims::from_header(request.headers());
    let uuid = request.extract().await;
    match uuid {
        Ok(uuid) => {
            if claims.acc {
                match delete_user_by_uuid(uuid).await {
                    Ok(_) => Ok(StatusCode::OK),
                    Err(_) => Err(AuthError::from_error_type(AuthErrorType::UserDoesNotExist)),
                }
            } else {
                Err(AuthError::from_error_type(AuthErrorType::AccessDenied))
            }
        }
        Err(error) => {
            println!("{error}");
            Err(AuthError::from_error_type(AuthErrorType::ServerError))
        }
    }
}
