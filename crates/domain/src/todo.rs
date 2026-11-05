use async_trait::async_trait;

use crate::error::RepositoryResult;

/// Priority levels an item may have: `(level, name)`.
pub const PRIORITY_LEVELS: [(i32, &str); 4] = [(0, "None"), (1, "Low"), (2, "Medium"), (3, "High")];

/// Colours a list may use: `(code, name)`. The first one is the default.
pub const COLOURS: [(&str, &str); 8] = [
    ("#FFFFFF", "White"),
    ("#FF5733", "Red"),
    ("#FFC300", "Orange"),
    ("#FFFF66", "Yellow"),
    ("#CCFF99", "Green"),
    ("#6666FF", "Blue"),
    ("#9966CC", "Purple"),
    ("#999999", "Grey"),
];

pub const MAX_TITLE_LEN: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TodoItem {
    pub id: i32,
    pub list_id: i32,
    pub title: Option<String>,
    pub done: bool,
    pub priority: i32,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TodoList {
    pub id: i32,
    pub title: Option<String>,
    pub colour: Option<String>,
    pub items: Vec<TodoItem>,
}

/// A title is required and limited to [`MAX_TITLE_LEN`] characters. Returns it trimmed.
pub fn normalize_title(title: Option<String>) -> Option<String> {
    title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty() && t.chars().count() <= MAX_TITLE_LEN)
}

/// Normalizes a colour code (case-insensitive) to one of the supported codes.
pub fn normalize_colour(colour: &str) -> Option<&'static str> {
    COLOURS
        .iter()
        .find(|(code, _)| code.eq_ignore_ascii_case(colour.trim()))
        .map(|(code, _)| *code)
}

pub fn is_valid_priority(priority: i32) -> bool {
    PRIORITY_LEVELS.iter().any(|(level, _)| *level == priority)
}

/// Storage of todo lists and items. Everything is scoped to the owning user (`user_id`), so a
/// record owned by someone else behaves as if it did not exist. The `bool` results tell whether a
/// record was affected.
#[async_trait]
pub trait TodoRepository: Send + Sync {
    /// All lists of the user (ordered by id) with their items (ordered by id).
    async fn lists_with_items(&self, user_id: i32) -> RepositoryResult<Vec<TodoList>>;
    async fn list_exists(&self, user_id: i32, id: i32) -> RepositoryResult<bool>;
    async fn insert_list(&self, user_id: i32, title: &str, colour: &str) -> RepositoryResult<i32>;
    /// A `None` colour keeps the current one.
    async fn update_list(
        &self,
        user_id: i32,
        id: i32,
        title: &str,
        colour: Option<&str>,
    ) -> RepositoryResult<bool>;
    /// Also removes the items of the list.
    async fn delete_list(&self, user_id: i32, id: i32) -> RepositoryResult<bool>;
    async fn insert_item(&self, list_id: i32, title: &str) -> RepositoryResult<i32>;
    async fn update_item(
        &self,
        user_id: i32,
        id: i32,
        title: &str,
        done: bool,
    ) -> RepositoryResult<bool>;
    async fn update_item_detail(
        &self,
        user_id: i32,
        id: i32,
        list_id: i32,
        priority: i32,
        note: Option<&str>,
    ) -> RepositoryResult<bool>;
    async fn delete_item(&self, user_id: i32, id: i32) -> RepositoryResult<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titles_are_trimmed_and_bounded() {
        assert_eq!(
            normalize_title(Some("  Buy milk ".into())).as_deref(),
            Some("Buy milk")
        );
        assert_eq!(normalize_title(None), None);
        assert_eq!(normalize_title(Some("   ".into())), None);
        assert!(normalize_title(Some("a".repeat(MAX_TITLE_LEN))).is_some());
        assert_eq!(normalize_title(Some("a".repeat(MAX_TITLE_LEN + 1))), None);
    }

    #[test]
    fn colours_match_case_insensitively() {
        assert_eq!(normalize_colour(" #ff5733 "), Some("#FF5733"));
        assert_eq!(normalize_colour("red"), None);
    }

    #[test]
    fn priority_must_be_a_known_level() {
        assert!(is_valid_priority(0) && is_valid_priority(3));
        assert!(!is_valid_priority(4) && !is_valid_priority(-1));
    }
}
