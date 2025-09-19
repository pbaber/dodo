use chrono::Local;
use crossterm::{event::{KeyCode, KeyEvent}};
use color_eyre::{Result};
use ratatui::{DefaultTerminal};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use std::env;
use crate::models::{TodoList, TodoItem, Status, InputMode, TodoRow, parse_date_string, new_todo_item};


mod ui;
mod models;

#[tokio::main]
async fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;

    let options = SqliteConnectOptions::from_str("sqlite:todos.db")?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    // Create the todos table if it doesn't exist
    sqlx::query(
        r#"
      CREATE TABLE IF NOT EXISTS todos (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          todo TEXT NOT NULL,
          details TEXT,
          status TEXT NOT NULL DEFAULT 'todo',
          date TEXT NOT NULL
      )
      "#
    )
        .execute(&pool)
    .await?;

    // We need to setup the app here because it's async
    // and we can't directly use the run method with what the async
    // function returns
    let app = App::with_pool(pool).await?;

    // Only run TUI if we're in a proper terminal environment
    match env::var("TERM") {
        Ok(_) => ratatui::run(|terminal| app.run(terminal))?,
        Err(_) => println!("Not running in a terminal, skipping TUI"),
    }

    Ok(())
}

struct App {
    should_exit: bool,
    pool: SqlitePool,
    todo_list: TodoList,
    input_mode: InputMode,
    character_index: usize,
    input: String,
}



impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Todo => write!(f, "todo"),
            Status::Completed => write!(f, "completed"),
        }
    }
}



impl App {
    async fn with_pool(pool: SqlitePool) -> Result<Self, sqlx::Error> {

        let rows = sqlx::query_as::<_, TodoRow>("SELECT id, todo, details, status, date FROM todos")
            .fetch_all(&pool)
            .await?;

        let todo_items: Vec<TodoItem> = rows.into_iter().map(|row| {
            TodoItem {
                id: Some(row.id),
                todo: row.todo,
                details: row.details,
                status: match row.status.as_str() {
                    "completed" => Status::Completed,
                    _ => Status::Todo,
                },
                date: parse_date_string(&row.date),
            }
        }).collect();


        let no_todos = {
            TodoList {
                items: vec![
                    TodoItem {
                        id: None,
                        todo: "Make a todo item".to_string(),
                        details: "One's life always has something to do".to_string(),
                        status: Status::Todo,
                        date: Local::now().date_naive(),
                    }
                ],
                state: ratatui::widgets::ListState::default(),
            }
        };

        Ok(Self {
            should_exit: false,
            pool,
            input_mode: InputMode::Normal,
            character_index: 0,
            input: String::from(""),
            todo_list: if todo_items.is_empty() {
                no_todos
            } else {
                TodoList {
                    items: todo_items,
                    state: ratatui::widgets::ListState::default(),
                }
            }
        })
    }
}

impl App {
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|f| self.render(f))?;
            
            if let Some(key) = crossterm::event::read()?.as_key_press_event() {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('i') => self.input_mode.toggle(),
                KeyCode::Char('q') => self.should_exit = true,
                KeyCode::Char('d') => self.delete_selected_todo(),
                KeyCode::Char('h') | KeyCode::Left => self.select_none(),
                KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('g') | KeyCode::Home => self.select_first(),
                KeyCode::Char('G') | KeyCode::End => self.select_last(),
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                    self.toggle_status();
                }
                _ => {}
            }
            InputMode::Insert => match key.code {
                KeyCode::Esc => self.input_mode.toggle(),
                KeyCode::Enter => self.add_input_todo(),
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                _ => {}
            }
        }
    }


    const fn select_none(&mut self) {
        self.todo_list.state.select(None);
    }

    fn select_next(&mut self) {
        self.todo_list.state.select_next();
    }
    fn select_previous(&mut self) {
        self.todo_list.state.select_previous();
    }

    const fn select_first(&mut self) {
        self.todo_list.state.select_first();
    }

    const fn select_last(&mut self) {
        self.todo_list.state.select_last();
    }

    /// Changes the status of the selected list item
    fn toggle_status(&mut self) {

        let Some(index) = self.todo_list.state.selected() else { return };
        let Some(todo) = self.todo_list.items.get(index) else { return };

        let pool = self.pool.clone();
        let todo_id = todo.id;

        self.todo_list.items[index].status = match todo.status {
            Status::Completed => Status::Todo,
            Status::Todo => Status::Completed,
        };

        tokio::spawn(async move {
            if let Err(e) = toggle_todo_status_in_database(&pool, todo_id).await {
                eprintln!("Database error toggling status: {}", e);
            }
        });

    }
}



impl App {
    fn add_input_todo(&mut self)  {
        let todo_item = new_todo_item(&self.input, "New status");

        let pool = self.pool.clone();
        let item_for_db = todo_item.clone();

        tokio::spawn(async move {
            if let Err(e) = write_input_to_database(&pool, &item_for_db).await {
                eprintln!("Database error: {}", e);
            }
        });

        self.todo_list.items.push(todo_item);
        self.input = String::new();
        self.character_index = 0;
    }

}

// Database
async fn write_input_to_database(pool: &SqlitePool, todo: &TodoItem) -> Result<(), sqlx::Error> {
    let query = "INSERT INTO todos (todo, details, status, date) VALUES (?, ?, ?, ?)";

    sqlx::query(query)
        .bind(&todo.todo)
        .bind(&todo.details)
        .bind(&todo.status.to_string())
        .bind(&todo.date.format("%Y-%m-%d").to_string())
        .execute(pool)
    .await?;

    return Ok(())
}

async fn delete_todo_from_database(pool: &SqlitePool, todo: &TodoItem) -> Result<(), sqlx::Error> {

    if let Some(id) = todo.id {
        sqlx::query("DELETE FROM todos WHERE id = ?")
            .bind(id)
            .execute(pool)
        .await?;
    }
    Ok(())
}

async fn toggle_todo_status_in_database(pool: &SqlitePool, todo_id: Option<i64>) -> Result<(), sqlx::Error> {
    if let Some(id) = todo_id {
        sqlx::query(
       r#"
        UPDATE todos SET status = CASE
        WHEN status = 'todo' THEN 'completed'
        WHEN status = 'completed' THEN 'todo'
        ELSE 'todo'
        END WHERE id = ?
        "#)
            .bind(id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

impl App {
    
    fn delete_selected_todo(&mut self) {
        if let Some(index) = self.todo_list.state.selected() {
            if index < self.todo_list.items.len() {
                let todo_to_delete = self.todo_list.items[index].clone();
                let pool = self.pool.clone();

                tokio::spawn(async move {
                    if let Err(e) = delete_todo_from_database(&pool,
                                    &todo_to_delete).await {
                        eprintln!("Database error deleting todo: {}", e);
                    }
                });

                self.todo_list.items.remove(index);

                if self.todo_list.items.is_empty() {
                    self.todo_list.state.select(None);
                } else if index >= self.todo_list.items.len() {
                    self.todo_list.state.select(Some(self.todo_list.items.len() - 1));
                }

            }
        }
    }

}



impl App {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);

            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
}
