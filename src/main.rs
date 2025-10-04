use color_eyre::Result;
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::env;
use std::str::FromStr;

mod app;
mod db;
mod models;
mod ui;

#[tokio::main]
async fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;

    let options = SqliteConnectOptions::from_str("sqlite:todos.db")?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    // Create the todos table if it doesn't exist
    crate::db::create_todos_table(&pool).await?;

    let app = crate::app::App::with_pool(pool).await?;

    match env::var("TERM") {
        Ok(_) => {
            terminal::enable_raw_mode()?;
            execute!(std::io::stdout(), EnterAlternateScreen)?;

            let backend = CrosstermBackend::new(std::io::stdout());
            let mut terminal = Terminal::new(backend)?;

            let result = app.run(&mut terminal);

            execute!(std::io::stdout(), LeaveAlternateScreen)?;

            terminal::disable_raw_mode()?;

            result?;
        }
        Err(_) => println!("Not running in a terminal, skipping TUI"),
    }

    Ok(())
}
