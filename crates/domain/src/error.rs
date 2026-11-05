use std::fmt;

/// Failure reported by a repository implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    /// A uniqueness constraint was violated (e.g. username or email already taken).
    Conflict,
    /// Any other storage failure; the message is for logging only.
    Unexpected(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict => write!(f, "conflicting record"),
            Self::Unexpected(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for RepositoryError {}

pub type RepositoryResult<T> = Result<T, RepositoryError>;
