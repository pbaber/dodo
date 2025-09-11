use chrono::Date
use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Color, Style},
};

struct App {
    should_exit: bool,
    todo_list: TodoList,
}

struct TodoList {
    items: Vec<TodoItem>,
    state: ListState,
}

#[derive(Debug)]
struct TodoItem {
    todo: String,
    date: Date,
    status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Status {
    Todo,
    Completed,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the terminal
    let mut terminal = ratatui::init();
    
    // Simple todo list data
    let todos = vec![
        "Learn Rust",
        "Build a TUI app",
        "Read documentation",
        "Write more code",
    ];
    
    let mut list_state = ListState::default().with_selected(Some(0));

    loop {
        // Draw the UI
        terminal.draw(|frame| {
            let area = frame.area();
            
            // Create list items from todo data
            let items: Vec<ListItem> = todos
                .iter()
                .map(|todo| ListItem::new(*todo))
                .collect();
            
            // Create the list widget
            let list = List::new(items)
                .block(Block::default().title("My Todo List").borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::Yellow))
                .highlight_symbol(">> ");
            
            // Render the list with state
            frame.render_stateful_widget(list, area, &mut list_state);
        })?;

        // Handle input
        if let Ok(event) = crossterm::event::read() {
            if let crossterm::event::Event::Key(key) = event {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Restore the terminal
    ratatui::restore();
    Ok(())
}
