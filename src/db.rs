use crate::models::TodoItem;
use crate::models::{TodoRow, parse_date_string, sort_todos_hierarchically};
use sqlx::sqlite::SqlitePool;

pub async fn create_todos_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
      CREATE TABLE IF NOT EXISTS todos (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          todo TEXT NOT NULL,
          details TEXT,
          date TEXT NOT NULL,
          completed_at TEXT NULL,
          parent_id INTEGER REFERENCES todos(id),
          sort_order INTEGER NOT NULL DEFAULT 0
      )
      "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn all_todos(pool: &SqlitePool) -> Result<Vec<TodoItem>, sqlx::Error> {
    let rows = sqlx::query_as::<_, TodoRow>(
        "SELECT id, todo, details, completed_at, date, parent_id, sort_order FROM todos ORDER BY sort_order",
    )
        .fetch_all(pool)
        .await?;

    let todo_items: Vec<TodoItem> = rows
        .into_iter()
        .map(|row| TodoItem {
            id: Some(row.id),
            todo: row.todo,
            details: row.details,
            completed_at: if row.completed_at.is_empty() {
                None
            } else {
                Some(parse_date_string(&row.completed_at))
            },
            date: parse_date_string(&row.date),
            parent_id: row.parent_id,
            sort_order: row.sort_order,
        })
        .collect();

    Ok(sort_todos_hierarchically(todo_items))
}

pub async fn uncompleted_todos(pool: &SqlitePool) -> Result<Vec<TodoItem>, sqlx::Error> {
    let uncompleted_todos: Vec<TodoItem> = all_todos(pool)
        .await?
        .into_iter()
        .filter(|item| item.completed_at.is_none())
        .collect();

    Ok(uncompleted_todos)
}

pub async fn completed_todos(pool: &SqlitePool) -> Result<Vec<TodoItem>, sqlx::Error> {
    let completed_todos: Vec<TodoItem> = all_todos(pool)
        .await?
        .into_iter()
        .filter(|item| item.completed_at.is_some())
        .collect();

    Ok(completed_todos)
}

pub async fn write_input_to_database(
    pool: &SqlitePool,
    todo: &TodoItem,
) -> Result<(), sqlx::Error> {
    let query = "INSERT INTO todos (todo, details, date, completed_at, parent_id, sort_order) VALUES (?, ?, ?, ?, ?, ?)";

    sqlx::query(query)
        .bind(&todo.todo)
        .bind(&todo.details)
        .bind(&todo.date.format("%Y-%m-%d %H:%M:%S").to_string())
        .bind(
            &todo
                .completed_at
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
        )
        .bind(&todo.parent_id)
        .bind(&todo.sort_order)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_todo_from_database(
    pool: &SqlitePool,
    todo: &TodoItem,
) -> Result<(), sqlx::Error> {
    if let Some(id) = todo.id {
        sqlx::query("DELETE FROM todos WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn update_todo_text(
    pool: &SqlitePool,
    todo_id: i64,
    new_text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE todos SET todo = ? WHERE id = ?")
        .bind(new_text)
        .bind(todo_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn update_todo_sort_order(
    pool: &SqlitePool,
    todo_id: i64,
    new_sort_order: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE todos SET sort_order = ? WHERE id = ?")
        .bind(new_sort_order)
        .bind(todo_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn toggle_todo_status_in_database(
    pool: &SqlitePool,
    todo_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    use chrono::Local;

    if let Some(id) = todo_id {
        let now = Local::now()
            .naive_local()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        sqlx::query(
            r#"
        UPDATE todos SET 
            completed_at = CASE
                WHEN completed_at IS NULL THEN ?
                ELSE NULL
            END
        WHERE id = ?
        "#,
        )
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    }
    Ok(())
}
