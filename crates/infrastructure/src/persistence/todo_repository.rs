use async_trait::async_trait;
use domain::{
    error::RepositoryResult,
    todo::{TodoItem, TodoList, TodoRepository},
};
use sqlx::{Row, any::AnyRow};

use super::unexpected;
use crate::db::DbPool;

pub struct SqlxTodoRepository {
    pool: DbPool,
}

impl SqlxTodoRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn item_from_row(row: &AnyRow) -> Result<TodoItem, sqlx::Error> {
    Ok(TodoItem {
        id: row.try_get("id")?,
        list_id: row.try_get("list_id")?,
        title: row.try_get("title")?,
        done: row.try_get("done")?,
        priority: row.try_get("priority")?,
        note: row.try_get("note")?,
    })
}

fn list_from_row(row: &AnyRow) -> Result<TodoList, sqlx::Error> {
    Ok(TodoList {
        id: row.try_get("id")?,
        title: row.try_get("title")?,
        colour: row.try_get("colour")?,
        items: vec![],
    })
}

#[async_trait]
impl TodoRepository for SqlxTodoRepository {
    async fn lists_with_items(&self, user_id: i32) -> RepositoryResult<Vec<TodoList>> {
        let list_rows = sqlx::query(
            "SELECT id, title, colour FROM \"TodoLists\" WHERE user_id = $1 ORDER BY id;",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(unexpected)?;
        let item_rows = sqlx::query(
            "SELECT i.id, i.list_id, i.title, i.done, i.priority, i.note FROM \"TodoItems\" i
             JOIN \"TodoLists\" l ON l.id = i.list_id WHERE l.user_id = $1 ORDER BY i.id;",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(unexpected)?;

        let mut lists = list_rows
            .iter()
            .map(list_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(unexpected)?;
        for row in &item_rows {
            let item = item_from_row(row).map_err(unexpected)?;
            if let Some(list) = lists.iter_mut().find(|l| l.id == item.list_id) {
                list.items.push(item);
            }
        }
        Ok(lists)
    }

    async fn list_exists(&self, user_id: i32, id: i32) -> RepositoryResult<bool> {
        Ok(
            sqlx::query("SELECT id FROM \"TodoLists\" WHERE id = $1 AND user_id = $2;")
                .bind(id)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(unexpected)?
                .is_some(),
        )
    }

    async fn insert_list(&self, user_id: i32, title: &str, colour: &str) -> RepositoryResult<i32> {
        let row = sqlx::query(
            "INSERT INTO \"TodoLists\" (title, colour, user_id) VALUES ($1, $2, $3) RETURNING id;",
        )
        .bind(title)
        .bind(colour)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(unexpected)?;
        row.try_get("id").map_err(unexpected)
    }

    async fn update_list(
        &self,
        user_id: i32,
        id: i32,
        title: &str,
        colour: Option<&str>,
    ) -> RepositoryResult<bool> {
        let result = sqlx::query("UPDATE \"TodoLists\" SET title = $2, colour = COALESCE($3, colour) WHERE id = $1 AND user_id = $4;")
            .bind(id)
            .bind(title)
            .bind(colour)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(unexpected)?;
        Ok(result.rows_affected() > 0)
    }

    /// The items are removed by the `ON DELETE CASCADE` foreign key.
    async fn delete_list(&self, user_id: i32, id: i32) -> RepositoryResult<bool> {
        let result = sqlx::query("DELETE FROM \"TodoLists\" WHERE id = $1 AND user_id = $2;")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(unexpected)?;
        Ok(result.rows_affected() > 0)
    }

    async fn insert_item(&self, list_id: i32, title: &str) -> RepositoryResult<i32> {
        let row =
            sqlx::query("INSERT INTO \"TodoItems\" (list_id, title) VALUES ($1, $2) RETURNING id;")
                .bind(list_id)
                .bind(title)
                .fetch_one(&self.pool)
                .await
                .map_err(unexpected)?;
        row.try_get("id").map_err(unexpected)
    }

    async fn update_item(
        &self,
        user_id: i32,
        id: i32,
        title: &str,
        done: bool,
    ) -> RepositoryResult<bool> {
        let result = sqlx::query("UPDATE \"TodoItems\" SET title = $2, done = $3 WHERE id = $1 AND list_id IN (SELECT id FROM \"TodoLists\" WHERE user_id = $4);")
            .bind(id)
            .bind(title)
            .bind(done)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(unexpected)?;
        Ok(result.rows_affected() > 0)
    }

    async fn update_item_detail(
        &self,
        user_id: i32,
        id: i32,
        list_id: i32,
        priority: i32,
        note: Option<&str>,
    ) -> RepositoryResult<bool> {
        let result = sqlx::query("UPDATE \"TodoItems\" SET list_id = $2, priority = $3, note = $4 WHERE id = $1 AND list_id IN (SELECT id FROM \"TodoLists\" WHERE user_id = $5);")
            .bind(id)
            .bind(list_id)
            .bind(priority)
            .bind(note)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(unexpected)?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_item(&self, user_id: i32, id: i32) -> RepositoryResult<bool> {
        let result = sqlx::query("DELETE FROM \"TodoItems\" WHERE id = $1 AND list_id IN (SELECT id FROM \"TodoLists\" WHERE user_id = $2);")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(unexpected)?;
        Ok(result.rows_affected() > 0)
    }
}
