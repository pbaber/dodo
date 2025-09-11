use ratatui::widgets::Widget;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use chrono::{Local, NaiveDate};
use crossterm::{event::{KeyCode, KeyEvent}};
use color_eyre::Result;

fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::default().run(terminal))?;
    Ok(())
}

struct App {
    should_exit: bool,
    #[allow(dead_code)]
    todo_list: TodoList,
}

#[allow(dead_code)]
#[derive(Debug)]
struct TodoItem {
    todo: String,
    details: String,
    status: Status,
    date: NaiveDate
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Status {
    Todo,
    Completed,
}

#[allow(dead_code)]
struct TodoList {
    items: Vec<TodoItem>,
    state: ratatui::widgets::ListState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            todo_list: TodoList {
                items: vec![
                    TodoItem {
                        todo: "Go outside and touch grass".to_string(),
                        details: "A way not to be cooked up all day".to_string(),
                        status: Status::Todo,
                        date: Local::now().date_naive(),
                    }
                ],
                state: ratatui::widgets::ListState::default(),
            }
        }
    }
}

impl App {
    fn run(mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Some(key) = crossterm::event::read()?.as_key_press_event() {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            _ => {}
        }

    }

}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
       use ratatui::widgets::Paragraph;
        Paragraph::new("My Todo App").render(area, buf);
    }
}
