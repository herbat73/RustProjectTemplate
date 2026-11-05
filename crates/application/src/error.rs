use std::fmt;

use domain::error::RepositoryError;

/// Outcome of a failed use case. The presentation layer decides how to render each variant
/// (HTTP status, message, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    WrongCredentials,
    TokenCreation,
    UserAlreadyExists,
    UserDoesNotExist,
    InvalidToken,
    BadRequest,
    AccessDenied,
    MissingFields,
    InvalidEmail,
    ResetLinkInvalid,
    NotFound,
    /// Unexpected failure; the message is for logging only and is never shown to clients.
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Internal(message) => write!(f, "internal error: {message}"),
            other => write!(f, "{other:?}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<RepositoryError> for AppError {
    fn from(error: RepositoryError) -> Self {
        Self::Internal(error.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
