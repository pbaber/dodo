use ratatui::layout::{Layout, Constraint, Position};
use chrono::{Local, NaiveDate};
use crossterm::{event::{KeyCode, KeyEvent}};
use color_eyre::{Result};
use ratatui::{DefaultTerminal};
use ratatui::style::{Stylize, Color, Style, Modifier};
use ratatui::style::palette::tailwind::{SLATE};
use ratatui::widgets::{
    Block, List, Paragraph, ListItem, HighlightSpacing};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use std::env;

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

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

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Insert,
}

impl InputMode {
    fn toggle(&mut self) {
        *self = match self {
            InputMode::Normal => InputMode::Insert,
            InputMode::Insert => InputMode::Normal,
        }
    }
}

#[derive(Debug, Clone)]
struct TodoItem {
    todo: String,
    details: String,
    status: Status,
    date: NaiveDate
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Status {
    Todo,
    Completed,
}

struct TodoList {
    items: Vec<TodoItem>,
    state: ratatui::widgets::ListState,
}

#[derive(sqlx::FromRow)]
struct TodoRow {
    todo: String,
    details: String,
    status: String,
    date: String,
}


impl App {
    async fn with_pool(pool: SqlitePool) -> Result<Self, sqlx::Error> {

        let rows = sqlx::query_as::<_, TodoRow>("SELECT todo, details, status, date FROM todos")
            .fetch_all(&pool)
            .await?;

        let todo_items: Vec<TodoItem> = rows.into_iter().map(|row| {
            TodoItem {
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
        if let Some(i) = self.todo_list.state.selected() {
            self.todo_list.items[i].status = match self.todo_list.items[i].status {
                Status::Completed => Status::Todo,
                Status::Todo => Status::Completed,
            }
        }
    }
}

fn parse_date_string(date_str: &str) -> NaiveDate {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .unwrap_or_else(|_| Local::now().date_naive())
}

fn new_todo_item(todo: &str, details: &str) -> TodoItem {
    TodoItem {
        todo: todo.to_string(),
        details: details.to_string(),
        status: Status::Todo,
        date: Local::now().date_naive()
    }
}

impl App {
    fn add_input_todo(&mut self) {
        let todo_item = new_todo_item(&self.input, "New status");
        self.todo_list.items.push(todo_item);
        self.input = String::new();
        self.character_index = 0;
    }
}

impl App {
    fn render(&mut self, frame: &mut ratatui::Frame) {
        let main_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Max(self.todo_list.items.len() as u16),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);

        let [
        top_area, 
        mid_area, 
        input_area, 
        blank_area, 
        bottom_area
    ] = frame.area().layout(&main_layout);

        frame.render_widget(self.title(), top_area);

        let items_cloned = self.todo_list.items.clone();
        let list = App::todo_list(items_cloned);
        frame.render_stateful_widget(list, mid_area, &mut self.todo_list.state);

        frame.render_widget(self.input_line(), input_area);

        frame.render_widget(Paragraph::new(String::from("")), blank_area);

        frame.render_widget(self.footer(), bottom_area);

        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Insert => frame.set_cursor_position(Position::new(
                input_area.x + self.character_index as u16 + 1,
                input_area.y + 1,
            )),
        }
    }
}

impl App {
    fn title(&self) -> Paragraph {
        if self.input_mode == InputMode::Insert {
        Paragraph::new("Insert Mode")
            .bold()
            .style(Style::default().fg(Color::Green))
            .centered()
        } else {
        Paragraph::new("Normal Mode")
            .bold()
            .style(Style::default().fg(Color::Yellow))
            .centered()
        }
    }

    fn todo_list(items: Vec<TodoItem>) -> List<'static> {
            let todo_items: Vec<ListItem> = items
            .iter()
            .map(|todo_item| {
                if todo_item.status == Status::Todo {
                    ListItem::new(format!("✓ {}", todo_item.todo))
                } else {
                    ListItem::new(format!("☐ {}", todo_item.todo))
                }
            })
            .collect();

        return List::new(todo_items)
            .block(Block::new())
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always)
    }

    fn input_line(&mut self) -> Paragraph {
        Paragraph::new(self.input.clone()) 
            .block(Block::bordered().title_top("New Todo"))
    }

    fn footer(&mut self) -> Paragraph<'static> {
        Paragraph::new("Here's the bottom part")
            .centered()
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
