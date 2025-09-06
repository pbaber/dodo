use ratatui::{
    widgets::{Block, Borders, Paragraph},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the terminal
    let mut terminal = ratatui::init();

    loop {
        // Draw the UI
        terminal.draw(|frame| {
            let area = frame.area();
            
            let block = Block::default()
                .title("My Todo TUI")
                .borders(Borders::ALL);
                
            let paragraph = Paragraph::new("Welcome to your Todo TUI!\n\nPress 'q' to quit.")
                .block(block);
                
            frame.render_widget(paragraph, area);
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
