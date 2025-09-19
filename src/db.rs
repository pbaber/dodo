use sqlx::sqlite::SqlitePool;
use crate::models::TodoItem;

pub async fn create_todos_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
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
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn write_input_to_database(pool: &SqlitePool, todo: &TodoItem) -> Result<(), sqlx::Error> {
    let query = "INSERT INTO todos (todo, details, status, date) VALUES (?, ?, ?, ?)";
    
    sqlx::query(query)
        .bind(&todo.todo)
        .bind(&todo.details)
        .bind(&todo.status.to_string())
        .bind(&todo.date.format("%Y-%m-%d").to_string())
        .execute(pool)
    .await?;
    
    Ok(())
}

pub async fn delete_todo_from_database(pool: &SqlitePool, todo: &TodoItem) -> Result<(), sqlx::Error> {
    if let Some(id) = todo.id {
        sqlx::query("DELETE FROM todos WHERE id = ?")
            .bind(id)
            .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn toggle_todo_status_in_database(pool: &SqlitePool, todo_id: Option<i64>) -> Result<(), sqlx::Error> {
    if let Some(id) = todo_id {
        sqlx::query(
       r#"
        UPDATE todos SET status = CASE
        WHEN status = 'todo' THEN 'completed'
        WHEN status = 'completed' THEN 'todo'
        ELSE 'todo'
        END WHERE id = ?
        "#)
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}