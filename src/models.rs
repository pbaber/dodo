use chrono::{Local, NaiveDateTime};

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
}

impl InputMode {
    pub fn toggle(&mut self) {
        *self = match self {
            InputMode::Normal => InputMode::Insert,
            InputMode::Insert => InputMode::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub id: Option<i64>,
    pub todo: String,
    pub details: String,
    pub completed_at: Option<NaiveDateTime>,
    pub date: NaiveDateTime,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
}

pub struct TodoList {
    pub items: Vec<TodoItem>,
    pub state: ratatui::widgets::ListState,
}

#[derive(sqlx::FromRow)]
pub struct TodoRow {
    pub id: i64,
    pub todo: String,
    pub details: String,
    pub completed_at: String,
    pub date: String,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
}

pub fn parse_date_string(date_str: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| Local::now().naive_local())
}

pub fn new_todo_item(todo: &str, details: &str, parent_id: Option<i64>) -> TodoItem {
    TodoItem {
        id: None,
        todo: todo.to_string(),
        details: details.to_string(),
        completed_at: None,
        date: Local::now().naive_local(),
        parent_id: parent_id,
        sort_order: 0,
    }
}

pub fn sort_todos_hierarchically(items: Vec<TodoItem>) -> Vec<TodoItem> {
    let mut result = Vec::new();

    let mut top_level: Vec<TodoItem> = items
        .iter()
        .filter(|item| item.parent_id.is_none())
        .cloned()
        .collect();

    top_level.sort_by_key(|item| item.sort_order);

    let children: Vec<TodoItem> = items
        .iter()
        .filter(|item| item.parent_id.is_some())
        .cloned()
        .collect();

    for parent in top_level {
        result.push(parent.clone());

        let mut parent_children: Vec<TodoItem> = children
            .iter()
            .filter(|child| child.parent_id == parent.id)
            .cloned()
            .collect();

        parent_children.sort_by_key(|item| item.sort_order);

        result.extend(parent_children);
    }

    result
}
