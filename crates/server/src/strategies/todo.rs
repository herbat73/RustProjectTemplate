use sqlx::{Row, any::AnyRow};
use types::todo::{TodoItemDto, TodoListDto};

use crate::pool;

fn item_from_row(row: &AnyRow) -> Result<TodoItemDto, sqlx::Error> {
    Ok(TodoItemDto {
        id: row.try_get("id")?,
        list_id: row.try_get("list_id")?,
        title: row.try_get("title")?,
        done: row.try_get("done")?,
        priority: row.try_get("priority")?,
        note: row.try_get("note")?,
    })
}

/// All lists of the user (ordered by id) with their items (ordered by id).
pub async fn get_lists_with_items(user_id: i32) -> Result<Vec<TodoListDto>, sqlx::Error> {
    let pool = pool::get_pool();
    let list_rows =
        sqlx::query("SELECT id, title, colour FROM \"TodoLists\" WHERE user_id = $1 ORDER BY id;")
            .bind(user_id)
            .fetch_all(&pool)
            .await?;
    let item_rows = sqlx::query(
        "SELECT i.id, i.list_id, i.title, i.done, i.priority, i.note FROM \"TodoItems\" i
         JOIN \"TodoLists\" l ON l.id = i.list_id WHERE l.user_id = $1 ORDER BY i.id;",
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await?;

    let mut lists = Vec::with_capacity(list_rows.len());
    for row in &list_rows {
        lists.push(TodoListDto {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            colour: row.try_get("colour")?,
            items: vec![],
        });
    }
    for row in &item_rows {
        let item = item_from_row(row)?;
        if let Some(list) = lists.iter_mut().find(|l| l.id == item.list_id) {
            list.items.push(item);
        }
    }
    Ok(lists)
}

/// Whether the list exists and belongs to the user.
pub async fn list_exists(user_id: i32, id: i32) -> Result<bool, sqlx::Error> {
    Ok(
        sqlx::query("SELECT id FROM \"TodoLists\" WHERE id = $1 AND user_id = $2;")
            .bind(id)
            .bind(user_id)
            .fetch_optional(&pool::get_pool())
            .await?
            .is_some(),
    )
}

pub async fn insert_list(user_id: i32, title: String, colour: String) -> Result<i32, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO \"TodoLists\" (title, colour, user_id) VALUES ($1, $2, $3) RETURNING id;",
    )
    .bind(title)
    .bind(colour)
    .bind(user_id)
    .fetch_one(&pool::get_pool())
    .await?;
    row.try_get("id")
}

/// Returns whether a list was updated.
pub async fn update_list(
    user_id: i32,
    id: i32,
    title: String,
    colour: Option<String>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE \"TodoLists\" SET title = $2, colour = COALESCE($3, colour) WHERE id = $1 AND user_id = $4;")
        .bind(id)
        .bind(title)
        .bind(colour)
        .bind(user_id)
        .execute(&pool::get_pool())
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Deletes the list; its items are removed by the `ON DELETE CASCADE` foreign key.
pub async fn delete_list(user_id: i32, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM \"TodoLists\" WHERE id = $1 AND user_id = $2;")
        .bind(id)
        .bind(user_id)
        .execute(&pool::get_pool())
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn insert_item(list_id: i32, title: String) -> Result<i32, sqlx::Error> {
    let row =
        sqlx::query("INSERT INTO \"TodoItems\" (list_id, title) VALUES ($1, $2) RETURNING id;")
            .bind(list_id)
            .bind(title)
            .fetch_one(&pool::get_pool())
            .await?;
    row.try_get("id")
}

pub async fn update_item(
    user_id: i32,
    id: i32,
    title: String,
    done: bool,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE \"TodoItems\" SET title = $2, done = $3 WHERE id = $1 AND list_id IN (SELECT id FROM \"TodoLists\" WHERE user_id = $4);")
        .bind(id)
        .bind(title)
        .bind(done)
        .bind(user_id)
        .execute(&pool::get_pool())
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_item_detail(
    user_id: i32,
    id: i32,
    list_id: i32,
    priority: i32,
    note: Option<String>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE \"TodoItems\" SET list_id = $2, priority = $3, note = $4 WHERE id = $1 AND list_id IN (SELECT id FROM \"TodoLists\" WHERE user_id = $5);")
        .bind(id)
        .bind(list_id)
        .bind(priority)
        .bind(note)
        .bind(user_id)
        .execute(&pool::get_pool())
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_item(user_id: i32, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM \"TodoItems\" WHERE id = $1 AND list_id IN (SELECT id FROM \"TodoLists\" WHERE user_id = $2);")
        .bind(id)
        .bind(user_id)
        .execute(&pool::get_pool())
        .await?;
    Ok(result.rows_affected() > 0)
}
