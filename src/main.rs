use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use chrono::{Local, NaiveDate};
use crossterm::{event::{KeyCode, KeyEvent}};
use color_eyre::Result;
use ratatui::{DefaultTerminal};
use ratatui::widgets::{
    Block, List, StatefulWidget, Paragraph, Widget, ListItem,
};

fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::default().run(terminal))?;
    Ok(())
}

struct App {
    should_exit: bool,
    todo_list: TodoList,
}

#[derive(Debug)]
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
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
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
            KeyCode::Enter => self.random_new_todo(),
            _ => {}
        }

    }

    fn random_new_todo(&mut self) {
        let new_item = random_new_todo_item();
        self.todo_list.items.push(new_item);
    }
}

fn new_todo_item(todo: &str, details: &str) -> TodoItem {
    TodoItem {
        todo: todo.to_string(),
        details: details.to_string(),
        status: Status::Todo,
        date: Local::now().date_naive()
    }
}

fn random_new_todo_item() -> TodoItem {
    new_todo_item("Hi there", "Another todo")
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);

        let [top_area, mid_area, bottom_area] = area.layout(&main_layout);

        App::render_top(top_area, buf);
        App::render_mid(self, mid_area, buf);
        App::render_bottom(bottom_area, buf);
    }
}

impl App {
    fn render_top(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Here's my app")
            .centered()
            .render(area, buf);
    }

    fn render_mid(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .todo_list
            .items
            .iter()
            .map(|todo_item| {
               ListItem::new(format!("☐ {}", todo_item.todo))
            })
            .collect();


        let list = List::new(items)
            .block(Block::new());

        StatefulWidget::render(list, area, buf, &mut self.todo_list.state);
    }

    fn render_bottom(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Here's the bottom part")
            .centered()
            .render(area, buf);
    }
}
