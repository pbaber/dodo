use ratatui::layout::{Layout, Constraint, Position};
use chrono::{Local, NaiveDate};
use crossterm::{event::{KeyCode, KeyEvent}};
use color_eyre::{owo_colors::OwoColorize, Result};
use ratatui::{DefaultTerminal};
use ratatui::style::{Stylize, Color, Style};
use ratatui::widgets::{
    Block, List, Paragraph, ListItem};

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
            KeyCode::Char('q') => self.should_exit = true,
            KeyCode::Enter => {
                self.todo_list.items.push(new_todo_item(&self.input, &String::from("nothing to see")))
            },
            KeyCode::Esc => self.input_mode = InputMode::Normal,
            KeyCode::Char('i') => self.input_mode.toggle(), 
            _ => {}
        }

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
                ListItem::new(format!("☐ {}", todo_item.todo))
            })
            .collect();

        return List::new(todo_items).block(Block::new())
    }

    fn input_line(&mut self) -> Paragraph {
        Paragraph::new(self.input.clone()) 
            .block(Block::bordered())
    }

    fn footer(&mut self) -> Paragraph<'static> {
        Paragraph::new("Here's the bottom part")
            .centered()
    }
}
