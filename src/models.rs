use chrono::{Local, NaiveDate};

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
    pub status: Status,
    pub completed_at: Option<NaiveDate>,
    pub date: NaiveDate,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    Todo,
    Completed,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Todo => write!(f, "todo"),
            Status::Completed => write!(f, "completed"),
        }
    }
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
    pub status: String,
    pub completed_at: String,
    pub date: String,
    pub sort_order: i32,
}

pub fn parse_date_string(date_str: &str) -> NaiveDate {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap_or_else(|_| Local::now().date_naive())
}

pub fn new_todo_item(todo: &str, details: &str) -> TodoItem {
    TodoItem {
        id: None,
        todo: todo.to_string(),
        details: details.to_string(),
        status: Status::Todo,
        completed_at: None,
        date: Local::now().date_naive(),
        sort_order: 0,
    }
}
