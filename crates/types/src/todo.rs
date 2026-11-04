use serde::{Deserialize, Serialize};

/// Priority levels accepted by `UpdateTodoItemDetailCommand.priority` (0..=3).
pub const PRIORITY_LEVELS: [(i32, &str); 4] = [(0, "None"), (1, "Low"), (2, "Medium"), (3, "High")];

/// Colours a todo list may use: `(code, name)`.
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct LookupDto {
    pub id: i32,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ColourDto {
    pub code: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TodoItemDto {
    pub id: i32,
    pub list_id: i32,
    pub title: Option<String>,
    pub done: bool,
    pub priority: i32,
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TodoListDto {
    pub id: i32,
    pub title: Option<String>,
    pub colour: Option<String>,
    pub items: Vec<TodoItemDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TodosVm {
    pub priority_levels: Vec<LookupDto>,
    pub colours: Vec<ColourDto>,
    pub lists: Vec<TodoListDto>,
}

impl TodosVm {
    pub fn new(lists: Vec<TodoListDto>) -> Self {
        Self {
            priority_levels: PRIORITY_LEVELS
                .iter()
                .map(|(id, title)| LookupDto { id: *id, title: Some((*title).to_string()) })
                .collect(),
            colours: COLOURS
                .iter()
                .map(|(code, name)| ColourDto { code: (*code).to_string(), name: (*name).to_string() })
                .collect(),
            lists,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateTodoListCommand {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub colour: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateTodoListCommand {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub colour: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CreateTodoItemCommand {
    #[serde(default)]
    pub list_id: i32,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateTodoItemCommand {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub done: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct UpdateTodoItemDetailCommand {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub list_id: i32,
    /// Priority level, see `TodosVm.priorityLevels` (0 = None .. 3 = High).
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_deserialize_camel_case_with_missing_fields() {
        let c: UpdateTodoItemDetailCommand = serde_json::from_str(r#"{"id":3,"listId":2,"priority":1}"#).unwrap();
        assert_eq!((c.id, c.list_id, c.priority, c.note), (3, 2, 1, None));
        let c: CreateTodoListCommand = serde_json::from_str("{}").unwrap();
        assert!(c.title.is_none() && c.colour.is_none());
    }

    #[test]
    fn view_model_serializes_camel_case_with_lookups() {
        let vm = TodosVm::new(vec![]);
        let json = serde_json::to_value(&vm).unwrap();
        assert_eq!(json["priorityLevels"].as_array().unwrap().len(), 4);
        assert_eq!(json["colours"][0]["code"], "#FFFFFF");
        assert!(json["lists"].as_array().unwrap().is_empty());
    }
}
