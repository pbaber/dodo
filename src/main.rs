use color_eyre::Result;
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

    // Only run TUI if we're in a proper terminal environment
    match env::var("TERM") {
        Ok(_) => ratatui::run(|terminal| app.run(terminal))?,
        Err(_) => println!("Not running in a terminal, skipping TUI"),
    }

    Ok(())
}
