use application::{AppError, auth::Session};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use http::{HeaderMap, HeaderValue, header::AUTHORIZATION};
use types::{
    auth::AuthErrorBody,
    user::{LoginUser, RegisterUser, ResetUser, UserInfo},
};

use crate::{
    error::ApiError,
    extractors::{AccessUser, RequesterUser},
    mapping::user_info,
    state::AppState,
};

// route function to nest endpoints in router
pub fn routes() -> Router<AppState> {
    Router::new()
        // routes requiring an access token
        .nest("/test", Router::new().route("/", get(test_auth_route)))
        // routes requiring a requester token
        .nest(
            "/request",
            Router::new().route("/", get(request_auth_token)),
        )
        // routes that do not need a token
        .route("/login", post(login_user))
        .route("/register", post(register_user))
        .nest(
            "/reset",
            Router::new()
                .route("/", post(request_reset))
                .route("/{reset_key}", post(reset_password)),
        )
}

/// Puts a token in the `authorization` response header.
fn token_header(token: &str) -> Result<HeaderMap, ApiError> {
    let value = HeaderValue::from_str(token).map_err(|_| AppError::TokenCreation)?;
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, value);
    Ok(headers)
}

fn session_response(session: Session) -> Result<(StatusCode, HeaderMap, Json<UserInfo>), ApiError> {
    let headers = token_header(&session.requester_token)?;
    Ok((StatusCode::CREATED, headers, Json(user_info(session.user))))
}

/// Verifies that the supplied access token is valid.
#[utoipa::path(
    get,
    path = "/auth/test",
    tag = "auth",
    security(("access_token" = [])),
    responses(
        (status = 200, description = "Token is valid", body = String, content_type = "text/plain", example = json!("Auth verified")),
        (status = 403, description = "Missing or invalid token", body = AuthErrorBody),
    )
)]
pub async fn test_auth_route(
    AccessUser(claims): AccessUser,
) -> Result<(StatusCode, String), ApiError> {
    println!("Authentication claims: {:?}", claims);
    Ok((StatusCode::OK, "Auth verified".to_string()))
}

/// Exchanges a requester token for a short-lived access token (returned in the `authorization` header).
#[utoipa::path(
    get,
    path = "/auth/request",
    tag = "auth",
    security(("requester_token" = [])),
    responses(
        (status = 201, description = "New access token issued", headers(("authorization" = String, description = "Bearer JWT access token (short-lived)"))),
        (status = 403, description = "Missing or invalid requester token", body = AuthErrorBody),
    )
)]
pub async fn request_auth_token(
    State(state): State<AppState>,
    RequesterUser(user_uuid): RequesterUser,
) -> Result<(StatusCode, HeaderMap), ApiError> {
    let token = state
        .auth
        .issue_access_token(&user_uuid)
        .await
        .inspect_err(|error| println!("Error creating token for UUID {user_uuid}: {error:?}"))?;
    Ok((StatusCode::CREATED, token_header(&token)?))
}

// route for logging in user with provided LoginUser json
/// Logs in with a username (or email) and password.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginUser,
    responses(
        (status = 201, description = "Logged in; a requester token is returned in the `authorization` header", body = UserInfo,
            headers(("authorization" = String, description = "Bearer JWT requester token"))),
        (status = 401, description = "Wrong credentials", body = AuthErrorBody),
        (status = 404, description = "User does not exist", body = AuthErrorBody),
    )
)]
pub async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginUser>,
) -> Result<(StatusCode, HeaderMap, Json<UserInfo>), ApiError> {
    let session = state.auth.login(&payload.username, &payload.pass).await?;
    session_response(session)
}

// handler for creating a new user
/// Registers a new user.
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = RegisterUser,
    responses(
        (status = 201, description = "User created; a requester token is returned in the `authorization` header", body = UserInfo,
            headers(("authorization" = String, description = "Bearer JWT requester token"))),
        (status = 400, description = "Missing fields or invalid email", body = AuthErrorBody),
        (status = 409, description = "Username or email already taken", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUser>,
) -> Result<(StatusCode, HeaderMap, Json<UserInfo>), ApiError> {
    let session = state
        .auth
        .register(&payload.username, &payload.pass, &payload.email)
        .await
        .inspect_err(|error| println!("Error creating user: {error}"))?;
    session_response(session)
}

/// Requests a password reset email. The request body is the plain-text email address.
#[utoipa::path(
    post,
    path = "/auth/reset",
    tag = "auth",
    request_body(content = String, content_type = "text/plain", description = "Email address of the account", example = json!("user@example.com")),
    responses(
        (status = 201, description = "Reset email sent"),
        (status = 400, description = "Invalid email address", body = AuthErrorBody),
        (status = 404, description = "User does not exist", body = AuthErrorBody),
        (status = 500, description = "Server or mail configuration error", body = AuthErrorBody),
    )
)]
pub async fn request_reset(
    State(state): State<AppState>,
    email_address: String,
) -> Result<StatusCode, ApiError> {
    state.auth.request_password_reset(&email_address).await?;
    Ok(StatusCode::CREATED)
}

/// Sets a new password using the key from the reset email.
#[utoipa::path(
    post,
    path = "/auth/reset/{reset_key}",
    tag = "auth",
    params(("reset_key" = String, Path, description = "Key from the password reset email")),
    request_body = ResetUser,
    responses(
        (status = 202, description = "Password updated"),
        (status = 400, description = "Reset link is invalid or expired", body = AuthErrorBody),
        (status = 404, description = "User does not exist", body = AuthErrorBody),
        (status = 500, description = "Server error", body = AuthErrorBody),
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Path(reset_key): Path<String>,
    Json(reset_user): Json<ResetUser>,
) -> Result<StatusCode, ApiError> {
    state
        .auth
        .reset_password(
            &reset_key,
            reset_user.email_address.as_ref(),
            &reset_user.pass,
        )
        .await?;
    Ok(StatusCode::ACCEPTED)
}
