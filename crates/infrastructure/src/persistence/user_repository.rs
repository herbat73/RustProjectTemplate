use async_trait::async_trait;
use domain::{
    error::{RepositoryError, RepositoryResult},
    user::{NewUser, User, UserRepository},
};
use sqlx::{Row, any::AnyRow};

use super::unexpected;
use crate::db::DbPool;

pub struct SqlxUserRepository {
    pool: DbPool,
}

impl SqlxUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn user_from_row(row: &AnyRow) -> Result<User, sqlx::Error> {
    Ok(User {
        id: row.try_get("id")?,
        uuid: row.try_get("uuid")?,
        username: row.try_get("username")?,
        password_hash: row.try_get("pass")?,
        email: row.try_get("email")?,
        is_admin: row.try_get("is_admin")?,
    })
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|e| e.is_unique_violation())
        || {
            let message = error.to_string();
            message.contains("duplicate key") || message.contains("UNIQUE constraint")
        }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn find_by_username_or_email(&self, value: &str) -> RepositoryResult<Option<User>> {
        sqlx::query("SELECT * FROM \"users\" WHERE username = $1 OR email = $1;")
            .bind(value)
            .fetch_optional(&self.pool)
            .await
            .map_err(unexpected)?
            .as_ref()
            .map(user_from_row)
            .transpose()
            .map_err(unexpected)
    }

    async fn find_by_uuid(&self, uuid: &str) -> RepositoryResult<Option<User>> {
        sqlx::query("SELECT * FROM \"users\" WHERE uuid = $1;")
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(unexpected)?
            .as_ref()
            .map(user_from_row)
            .transpose()
            .map_err(unexpected)
    }

    async fn list(&self) -> RepositoryResult<Vec<User>> {
        sqlx::query("SELECT * FROM \"users\";")
            .fetch_all(&self.pool)
            .await
            .map_err(unexpected)?
            .iter()
            .map(user_from_row)
            .collect::<Result<_, _>>()
            .map_err(unexpected)
    }

    async fn insert(&self, user: NewUser) -> RepositoryResult<User> {
        let row = sqlx::query(
            "INSERT INTO \"users\" (uuid, username, pass, email, is_admin)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *;",
        )
        .bind(user.uuid)
        .bind(user.username)
        .bind(user.password_hash)
        .bind(user.email)
        .bind(false)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| {
            if is_unique_violation(&error) {
                RepositoryError::Conflict
            } else {
                unexpected(error)
            }
        })?;
        user_from_row(&row).map_err(unexpected)
    }

    async fn update_password_hash(&self, id: i32, password_hash: &str) -> RepositoryResult<()> {
        sqlx::query("UPDATE \"users\" SET pass = $2 WHERE id = $1;")
            .bind(id)
            .bind(password_hash)
            .execute(&self.pool)
            .await
            .map_err(unexpected)?;
        Ok(())
    }

    async fn delete_by_uuid(&self, uuid: &str) -> RepositoryResult<bool> {
        let result = sqlx::query("DELETE FROM \"users\" WHERE uuid = $1;")
            .bind(uuid)
            .execute(&self.pool)
            .await
            .map_err(unexpected)?;
        Ok(result.rows_affected() > 0)
    }
}
