use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use chrono::{Local, NaiveDate};
use crossterm::{event::{KeyCode, KeyEvent}};
use color_eyre::Result;
use ratatui::{DefaultTerminal};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{
    Block, List, StatefulWidget, Paragraph, Widget, ListItem};

fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::default().run(terminal))?;
    Ok(())
}

struct App {
    should_exit: bool,
    todo_list: TodoList,
    input_mode: InputMode,
    character_index: usize,
    input: String,
}

enum InputMode {
    Normal,
    Insert,
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

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            input_mode: InputMode::Normal,
            character_index: 0,
            input: String::from("INPUT AREA"),
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
            terminal.draw(|f| self.render(f))?;
            
            if let Some(key) = crossterm::event::read()?.as_key_press_event() {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Enter => {
                self.todo_list.items.push(new_todo_item(&self.input, &String::from("nothing to see")))
            },
            _ => {}
        }

    }

    fn random_new_todo(&mut self) {
        let new_item = random_new_todo_item();
        self.todo_list.items.push(new_item);
        let last_index = self.todo_list.items.len().saturating_sub(1);
        self.todo_list.state.select(Some(last_index));
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


        // App::render_mid(self, mid_area, buf);
        // App::render_input_area(self, input_area, buf);
        // App::render_blank_area(self, blank_area, buf);
        // App::render_bottom(bottom_area, buf);
    }
}

impl App {
    fn title(&self) -> Paragraph {
        Paragraph::new("Here's my app")
            .bold()
            .centered()
    }

    fn todo_list(items: Vec<TodoItem>) -> List<'static> {
            let todo_items: Vec<ListItem> = items
            .iter()
            .map(|todo_item| {
                ListItem::new(format!("☐ {}", todo_item.todo))
            })
            .collect();

        return List::new(todo_items).block(Block::new())
    }

    fn render_input_area(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.input.clone()) 
            .block(Block::bordered())
            .render(area, buf);
    }

    fn render_blank_area(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(String::from(""))
            .render(area, buf);
    }

    fn render_bottom(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Here's the bottom part")
            .centered()
            .render(area, buf);
    }
}
