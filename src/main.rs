use color_eyre::Result;
use ratatui::backend;
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
            crossterm::terminal::enable_raw_mode()?;
            crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

            let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
            let mut terminal = ratatui::Terminal::new(backend)?;

            let result = app.run(&mut terminal);

            crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

            crossterm::terminal::disable_raw_mode()?;

            result?;
        }
        Err(_) => println!("Not running in a terminal, skipping TUI"),
    }

    Ok(())
}
