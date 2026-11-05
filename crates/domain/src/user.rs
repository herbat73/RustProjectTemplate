use async_trait::async_trait;

use crate::error::RepositoryResult;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: i32,
    pub uuid: String,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub is_admin: bool,
}

/// A user that has not been stored yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewUser {
    pub uuid: String,
    pub username: String,
    pub password_hash: String,
    pub email: String,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_username_or_email(&self, value: &str) -> RepositoryResult<Option<User>>;
    async fn find_by_uuid(&self, uuid: &str) -> RepositoryResult<Option<User>>;
    async fn list(&self) -> RepositoryResult<Vec<User>>;
    /// Stores a new (non-admin) user. Fails with `RepositoryError::Conflict` if the username or
    /// email is already taken.
    async fn insert(&self, user: NewUser) -> RepositoryResult<User>;
    async fn update_password_hash(&self, id: i32, password_hash: &str) -> RepositoryResult<()>;
    /// Returns whether a user was deleted.
    async fn delete_by_uuid(&self, uuid: &str) -> RepositoryResult<bool>;
}
