use chrono::Local;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{DefaultTerminal, widgets::ListState};
use sqlx::sqlite::SqlitePool;
use tui_textarea::TextArea;

use crate::models::{
    InputMode, TodoItem, TodoList, TodoRow, new_todo_item, parse_date_string,
    sort_todos_hierarchically,
};

pub struct App {
    pub should_exit: bool,
    pub pool: SqlitePool,
    pub todo_list: TodoList,
    pub creating_child_todo: bool,
    pub editing_index: Option<usize>,
    pub input_mode: InputMode,
    pub textarea: TextArea<'static>,
}

// Public API - Core Application Interface
impl App {
    /// Creates a new App instance with database connection and loads existing todos
    pub async fn with_pool(pool: SqlitePool) -> Result<Self, sqlx::Error> {
        let rows = sqlx::query_as::<_, TodoRow>(
            "SELECT id, todo, details, completed_at, date, parent_id, sort_order FROM todos ORDER BY sort_order",
        )
        .fetch_all(&pool)
        .await?;

        let todo_items: Vec<TodoItem> = rows
            .into_iter()
            .map(|row| TodoItem {
                id: Some(row.id),
                todo: row.todo,
                details: row.details,
                completed_at: if row.completed_at.is_empty() {
                    None
                } else {
                    Some(parse_date_string(&row.completed_at))
                },
                date: parse_date_string(&row.date),
                parent_id: row.parent_id,
                sort_order: row.sort_order,
            })
            .collect();

        let todo_items = sort_todos_hierarchically(todo_items);

        let no_todos = {
            TodoList {
                items: vec![TodoItem {
                    id: None,
                    todo: "Make a todo item".to_string(),
                    details: "One's life always has something to do".to_string(),
                    completed_at: None,
                    date: Local::now().naive_local(),
                    parent_id: None,
                    sort_order: 0,
                }],
                state: ListState::default(),
            }
        };

        Ok(Self {
            should_exit: false,
            pool,
            input_mode: InputMode::Normal,
            editing_index: None,
            creating_child_todo: false,
            todo_list: if todo_items.is_empty() {
                no_todos
            } else {
                TodoList {
                    items: todo_items,
                    state: ListState::default(),
                }
            },
            textarea: TextArea::default(),
        })
    }

    /// Main application loop that handles rendering and input
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|f| crate::ui::render_impl(&mut self, f))?;

            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    /// Handles keyboard input and routes to appropriate actions
    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('i') => {
                    self.creating_child_todo = false;
                    self.input_mode.toggle();
                }
                KeyCode::Char('o') => {
                    self.creating_child_todo = true;
                    self.input_mode.toggle();
                }
                KeyCode::Char('e') => self.enter_edit_mode(),
                KeyCode::Char('q') => self.should_exit = true,
                KeyCode::Char('d') => self.delete_selected_todo(),
                KeyCode::Char('h') | KeyCode::Left => self.select_none(),
                KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                KeyCode::Char('G') | KeyCode::End => self.select_last(),
                KeyCode::Char('J') => self.move_todo_down(),
                KeyCode::Char('K') => self.move_todo_up(),
                KeyCode::Char('c') | KeyCode::Right | KeyCode::Enter => {
                    self.toggle_status();
                }
                _ => {}
            },
            InputMode::Insert => match key.code {
                KeyCode::Esc => {
                    self.editing_index = None;
                    self.input_mode.toggle();
                    self.textarea = TextArea::default();
                }
                KeyCode::Enter => {
                    if self.editing_index.is_some() {
                        self.save_edited_todo();
                    } else {
                        self.add_input_todo();
                    }
                }
                _ => {
                    let input = tui_textarea::Input::from(key);
                    self.textarea.input(input);
                }
            },
        }
    }

    pub fn move_todo_up(&mut self) {
        if let Some(index) = self.todo_list.state.selected() {
            if index > 0 && index < self.todo_list.items.len() {
                // Swap sort_orders between current and previous item
                let current_order = self.todo_list.items[index].sort_order;
                let prev_order = self.todo_list.items[index - 1].sort_order;

                self.todo_list.items[index].sort_order = prev_order;
                self.todo_list.items[index - 1].sort_order = current_order;

                // Swap items in the list
                self.todo_list.items.swap(index, index - 1);
                self.todo_list.state.select(Some(index - 1));

                // Update database
                self.update_sort_orders_in_db();
            }
        }
    }

    pub fn move_todo_down(&mut self) {
        if let Some(index) = self.todo_list.state.selected() {
            if index < self.todo_list.items.len() - 1 {
                // Swap sort_orders between current and next item
                let current_order = self.todo_list.items[index].sort_order;
                let next_order = self.todo_list.items[index + 1].sort_order;

                self.todo_list.items[index].sort_order = next_order;
                self.todo_list.items[index + 1].sort_order = current_order;

                // Swap items in the list
                self.todo_list.items.swap(index, index + 1);
                self.todo_list.state.select(Some(index + 1));

                // Update database
                self.update_sort_orders_in_db();
            }
        }
    }

    fn update_sort_orders_in_db(&self) {
        let pool = self.pool.clone();
        let items = self.todo_list.items.clone();

        tokio::spawn(async move {
            for item in items {
                if let Some(id) = item.id {
                    if let Err(e) =
                        crate::db::update_todo_sort_order(&pool, id, item.sort_order).await
                    {
                        eprintln!("Database error updating sort order: {}", e);
                    }
                }
            }
        });
    }

    /// Selection methods for navigating the todo list
    pub fn select_none(&mut self) {
        self.todo_list.state.select(None);
    }

    pub fn select_next(&mut self) {
        self.todo_list.state.select_next();
    }

    pub fn select_previous(&mut self) {
        self.todo_list.state.select_previous();
    }

    pub fn select_first(&mut self) {
        self.todo_list.state.select_first();
    }

    pub fn select_last(&mut self) {
        self.todo_list.state.select_last();
    }
}

// Business Logic - Core Todo Operations
impl App {
    pub fn save_edited_todo(&mut self) {
        let Some(index) = self.editing_index else {
            return;
        };
        if index >= self.todo_list.items.len() {
            return;
        }

        let new_text = self.textarea.lines().join("\n");
        self.todo_list.items[index].todo = new_text.clone();

        let pool = self.pool.clone();
        let todo_id = self.todo_list.items[index].id;

        tokio::spawn(async move {
            if let Some(id) = todo_id {
                if let Err(e) = crate::db::update_todo_text(&pool, id, &new_text).await {
                    eprintln!("Database error updating todo text: {}", e);
                }
            }
        });

        self.textarea = TextArea::default();
        self.editing_index = None;
        self.input_mode.toggle();
    }

    pub fn enter_edit_mode(&mut self) {
        let Some(index) = self.todo_list.state.selected() else {
            return;
        };
        let Some(todo) = self.todo_list.items.get(index) else {
            return;
        };

        self.textarea = TextArea::new(vec![todo.todo.clone()]);
        self.editing_index = Some(index);
        self.input_mode.toggle();
    }

    /// Changes the status of the selected list item
    pub fn toggle_status(&mut self) {
        let Some(index) = self.todo_list.state.selected() else {
            return;
        };
        let Some(todo) = self.todo_list.items.get(index) else {
            return;
        };

        let pool = self.pool.clone();
        let todo_id = todo.id;

        self.todo_list.items[index].completed_at =
            if self.todo_list.items[index].completed_at.is_some() {
                None
            } else {
                Some(Local::now().naive_local())
            };

        let todo_item = self.todo_list.items[index].clone();

        tokio::spawn(async move {
            if let Err(e) =
                crate::db::toggle_todo_status_in_database(&pool, todo_id, &todo_item).await
            {
                eprintln!("Database error toggling status: {}", e);
            }
        });
    }

    /// Adds a new todo item from user input
    pub fn add_input_todo(&mut self) {
        let next_sort_order = self
            .todo_list
            .items
            .iter()
            .map(|item| item.sort_order)
            .max()
            .unwrap_or(0)
            + 10;

        let parent_id = if self.creating_child_todo {
            self.todo_list
                .state
                .selected()
                .and_then(|index| self.todo_list.items.get(index))
                .and_then(|item| item.id)
        } else {
            None
        };

        let input_text = self.textarea.lines().join("\n");
        let mut todo_item = new_todo_item(&input_text, "New Status", parent_id);
        todo_item.sort_order = next_sort_order;

        let pool = self.pool.clone();
        let item_for_db = todo_item.clone();

        tokio::spawn(async move {
            if let Err(e) = crate::db::write_input_to_database(&pool, &item_for_db).await {
                eprintln!("Database error: {}", e);
            }
        });

        self.todo_list.items.push(todo_item);
        self.todo_list.items = sort_todos_hierarchically(self.todo_list.items.clone());
        self.textarea = TextArea::default();
    }

    /// Deletes the currently selected todo item
    pub fn delete_selected_todo(&mut self) {
        if let Some(index) = self.todo_list.state.selected() {
            if index < self.todo_list.items.len() {
                let todo_to_delete = self.todo_list.items[index].clone();
                let pool = self.pool.clone();

                tokio::spawn(async move {
                    if let Err(e) =
                        crate::db::delete_todo_from_database(&pool, &todo_to_delete).await
                    {
                        eprintln!("Database error deleting todo: {}", e);
                    }
                });

                self.todo_list.items.remove(index);

                if self.todo_list.items.is_empty() {
                    self.todo_list.state.select(None);
                } else if index >= self.todo_list.items.len() {
                    self.todo_list
                        .state
                        .select(Some(self.todo_list.items.len() - 1));
                }
            }
        }
    }
}
