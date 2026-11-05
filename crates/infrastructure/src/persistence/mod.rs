mod todo_repository;
mod user_repository;

pub use todo_repository::SqlxTodoRepository;
pub use user_repository::SqlxUserRepository;

use domain::error::RepositoryError;

fn unexpected(error: sqlx::Error) -> RepositoryError {
    RepositoryError::Unexpected(error.to_string())
}
