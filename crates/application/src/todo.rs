use std::sync::Arc;

use domain::{
    todo::{
        COLOURS, TodoList, TodoRepository, is_valid_priority, normalize_colour, normalize_title,
    },
    user::UserRepository,
};

use crate::error::{AppError, AppResult};

/// Todo lists and items of one user. Every operation identifies the caller by user uuid; records
/// of other users are reported as not found.
pub struct TodoService {
    users: Arc<dyn UserRepository>,
    todos: Arc<dyn TodoRepository>,
}

impl TodoService {
    pub fn new(users: Arc<dyn UserRepository>, todos: Arc<dyn TodoRepository>) -> Self {
        Self { users, todos }
    }

    async fn user_id(&self, user_uuid: &str) -> AppResult<i32> {
        self.users
            .find_by_uuid(user_uuid)
            .await?
            .map(|user| user.id)
            .ok_or(AppError::UserDoesNotExist)
    }

    fn title(title: Option<String>) -> AppResult<String> {
        normalize_title(title).ok_or(AppError::MissingFields)
    }

    fn colour(colour: Option<String>) -> AppResult<Option<String>> {
        colour
            .map(|c| {
                normalize_colour(&c)
                    .map(str::to_string)
                    .ok_or(AppError::BadRequest)
            })
            .transpose()
    }

    pub async fn lists(&self, user_uuid: &str) -> AppResult<Vec<TodoList>> {
        let user_id = self.user_id(user_uuid).await?;
        Ok(self.todos.lists_with_items(user_id).await?)
    }

    /// Returns the id of the new list. Without a colour the default one is used.
    pub async fn create_list(
        &self,
        user_uuid: &str,
        title: Option<String>,
        colour: Option<String>,
    ) -> AppResult<i32> {
        let user_id = self.user_id(user_uuid).await?;
        let title = Self::title(title)?;
        let colour = Self::colour(colour)?.unwrap_or_else(|| COLOURS[0].0.to_string());
        Ok(self.todos.insert_list(user_id, &title, &colour).await?)
    }

    /// Without a colour the current one is kept.
    pub async fn update_list(
        &self,
        user_uuid: &str,
        id: i32,
        title: Option<String>,
        colour: Option<String>,
    ) -> AppResult<()> {
        let title = Self::title(title)?;
        let colour = Self::colour(colour)?;
        let user_id = self.user_id(user_uuid).await?;
        found(
            self.todos
                .update_list(user_id, id, &title, colour.as_deref())
                .await?,
        )
    }

    /// Also deletes the items of the list.
    pub async fn delete_list(&self, user_uuid: &str, id: i32) -> AppResult<()> {
        let user_id = self.user_id(user_uuid).await?;
        found(self.todos.delete_list(user_id, id).await?)
    }

    /// Returns the id of the new item.
    pub async fn create_item(
        &self,
        user_uuid: &str,
        list_id: i32,
        title: Option<String>,
    ) -> AppResult<i32> {
        let user_id = self.user_id(user_uuid).await?;
        let title = Self::title(title)?;
        found(self.todos.list_exists(user_id, list_id).await?)?;
        Ok(self.todos.insert_item(list_id, &title).await?)
    }

    pub async fn update_item(
        &self,
        user_uuid: &str,
        id: i32,
        title: Option<String>,
        done: bool,
    ) -> AppResult<()> {
        let title = Self::title(title)?;
        let user_id = self.user_id(user_uuid).await?;
        found(self.todos.update_item(user_id, id, &title, done).await?)
    }

    /// Moves the item to another list of the user and sets its priority and note.
    pub async fn update_item_detail(
        &self,
        user_uuid: &str,
        id: i32,
        list_id: i32,
        priority: i32,
        note: Option<String>,
    ) -> AppResult<()> {
        if !is_valid_priority(priority) {
            return Err(AppError::BadRequest);
        }
        let user_id = self.user_id(user_uuid).await?;
        found(self.todos.list_exists(user_id, list_id).await?)?;
        found(
            self.todos
                .update_item_detail(user_id, id, list_id, priority, note.as_deref())
                .await?,
        )
    }

    pub async fn delete_item(&self, user_uuid: &str, id: i32) -> AppResult<()> {
        let user_id = self.user_id(user_uuid).await?;
        found(self.todos.delete_item(user_id, id).await?)
    }
}

fn found(affected: bool) -> AppResult<()> {
    if affected {
        Ok(())
    } else {
        Err(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{FakeTodos, FakeUsers, user};

    fn service() -> TodoService {
        let users = FakeUsers::with(vec![
            user(1, "alice-uuid", "alice", false),
            user(2, "bob-uuid", "bob", false),
        ]);
        TodoService::new(Arc::new(users), Arc::new(FakeTodos::default()))
    }

    #[tokio::test]
    async fn create_list_defaults_the_colour_and_validates_input() {
        let todos = service();
        let id = todos
            .create_list("alice-uuid", Some("  Groceries ".into()), None)
            .await
            .unwrap();
        let lists = todos.lists("alice-uuid").await.unwrap();
        assert_eq!(lists.len(), 1);
        assert_eq!(
            (
                lists[0].id,
                lists[0].title.as_deref(),
                lists[0].colour.as_deref()
            ),
            (id, Some("Groceries"), Some("#FFFFFF"))
        );

        assert_eq!(
            todos.create_list("alice-uuid", None, None).await,
            Err(AppError::MissingFields)
        );
        assert_eq!(
            todos
                .create_list("alice-uuid", Some("x".into()), Some("pink".into()))
                .await,
            Err(AppError::BadRequest)
        );
        assert_eq!(
            todos.create_list("ghost", Some("x".into()), None).await,
            Err(AppError::UserDoesNotExist)
        );
    }

    #[tokio::test]
    async fn other_users_records_are_not_found() {
        let todos = service();
        let list = todos
            .create_list("alice-uuid", Some("Mine".into()), None)
            .await
            .unwrap();
        let item = todos
            .create_item("alice-uuid", list, Some("Milk".into()))
            .await
            .unwrap();

        assert_eq!(
            todos
                .create_item("bob-uuid", list, Some("Hack".into()))
                .await,
            Err(AppError::NotFound)
        );
        assert_eq!(
            todos
                .update_item("bob-uuid", item, Some("x".into()), true)
                .await,
            Err(AppError::NotFound)
        );
        assert_eq!(
            todos.delete_item("bob-uuid", item).await,
            Err(AppError::NotFound)
        );
        assert_eq!(
            todos.delete_list("bob-uuid", list).await,
            Err(AppError::NotFound)
        );

        todos
            .update_item("alice-uuid", item, Some("Oat milk".into()), true)
            .await
            .unwrap();
        let lists = todos.lists("alice-uuid").await.unwrap();
        assert_eq!(lists[0].items[0].title.as_deref(), Some("Oat milk"));
        assert!(lists[0].items[0].done);
    }

    #[tokio::test]
    async fn item_detail_checks_priority_and_target_list() {
        let todos = service();
        let a = todos
            .create_list("alice-uuid", Some("A".into()), None)
            .await
            .unwrap();
        let b = todos
            .create_list("alice-uuid", Some("B".into()), None)
            .await
            .unwrap();
        let other = todos
            .create_list("bob-uuid", Some("Bob's".into()), None)
            .await
            .unwrap();
        let item = todos
            .create_item("alice-uuid", a, Some("Milk".into()))
            .await
            .unwrap();

        assert_eq!(
            todos
                .update_item_detail("alice-uuid", item, b, 9, None)
                .await,
            Err(AppError::BadRequest)
        );
        assert_eq!(
            todos
                .update_item_detail("alice-uuid", item, other, 1, None)
                .await,
            Err(AppError::NotFound)
        );

        todos
            .update_item_detail("alice-uuid", item, b, 3, Some("soon".into()))
            .await
            .unwrap();
        let lists = todos.lists("alice-uuid").await.unwrap();
        assert!(lists[0].items.is_empty());
        assert_eq!(
            (
                lists[1].items[0].priority,
                lists[1].items[0].note.as_deref()
            ),
            (3, Some("soon"))
        );
    }
}
