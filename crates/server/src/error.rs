use application::AppError;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde_json::json;
use types::auth::{AuthError, AuthErrorType};

/// An [`AppError`] rendered as an HTTP response with the `AuthErrorBody` JSON body.
#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(error: AppError) -> Self {
        if let AppError::Internal(message) = &error {
            println!("{message}");
        }
        Self(error)
    }
}

fn error_type(error: &AppError) -> AuthErrorType {
    match error {
        AppError::WrongCredentials => AuthErrorType::WrongCredentials,
        AppError::TokenCreation => AuthErrorType::TokenCreation,
        AppError::UserAlreadyExists => AuthErrorType::UserAlreadyExists,
        AppError::UserDoesNotExist => AuthErrorType::UserDoesNotExist,
        AppError::InvalidToken => AuthErrorType::InvalidToken,
        AppError::BadRequest => AuthErrorType::BadRequest,
        AppError::AccessDenied => AuthErrorType::AccessDenied,
        AppError::MissingFields => AuthErrorType::MissingFields,
        AppError::InvalidEmail => AuthErrorType::InvalidEmail,
        AppError::ResetLinkInvalid => AuthErrorType::ResetLinkInvalid,
        AppError::NotFound => AuthErrorType::NotFound,
        AppError::Internal(_) => AuthErrorType::ServerError,
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let error = AuthError::from_error_type(error_type(&self.0));
        (error.status(), Json(json!(error.body()))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::StatusCode;

    #[test]
    fn errors_map_to_their_http_status() {
        for (error, status) in [
            (AppError::WrongCredentials, StatusCode::UNAUTHORIZED),
            (AppError::UserAlreadyExists, StatusCode::CONFLICT),
            (AppError::UserDoesNotExist, StatusCode::NOT_FOUND),
            (AppError::NotFound, StatusCode::NOT_FOUND),
            (AppError::InvalidToken, StatusCode::FORBIDDEN),
            (AppError::AccessDenied, StatusCode::FORBIDDEN),
            (AppError::MissingFields, StatusCode::BAD_REQUEST),
            (AppError::ResetLinkInvalid, StatusCode::BAD_REQUEST),
            (AppError::TokenCreation, StatusCode::INTERNAL_SERVER_ERROR),
            (
                AppError::Internal("db down".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ] {
            assert_eq!(ApiError(error).into_response().status(), status);
        }
    }

    #[test]
    fn internal_details_are_not_exposed() {
        let body = AuthError::from_error_type(error_type(&AppError::Internal("secret".into())));
        assert!(!body.body().message.contains("secret"));
    }
}
