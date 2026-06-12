use crate::models::Task;
use rusqlite::{params, Connection, Result};
use std::sync::Mutex;
use tauri::State;

pub fn list_tasks(conn: &Connection, status_filter: Option<String>) -> Result<Vec<Task>> {
    let sql = match status_filter {
        Some(_) => "SELECT id, title, description, quadrant, status, created_at, completed_at, updated_at FROM tasks WHERE status = ?1 ORDER BY updated_at DESC",
        None => "SELECT id, title, description, quadrant, status, created_at, completed_at, updated_at FROM tasks ORDER BY updated_at DESC",
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = match status_filter {
        Some(s) => stmt.query_map([s], row_to_task)?,
        None => stmt.query_map([], row_to_task)?,
    };
    rows.collect()
}

fn row_to_task(row: &rusqlite::Row) -> Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        quadrant: row.get(3)?,
        status: row.get(4)?,
        created_at: row.get(5)?,
        completed_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

pub fn create_task(
    conn: &Connection,
    title: &str,
    description: &str,
    quadrant: i32,
) -> Result<Task> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "INSERT INTO tasks (title, description, quadrant, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![title, description, quadrant, now, now],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Task {
        id,
        title: title.to_string(),
        description: description.to_string(),
        quadrant,
        status: "active".to_string(),
        created_at: now.clone(),
        completed_at: None,
        updated_at: now,
    })
}

pub fn update_task(
    conn: &Connection,
    id: i64,
    title: &str,
    description: &str,
    quadrant: i32,
) -> Result<()> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "UPDATE tasks SET title = ?1, description = ?2, quadrant = ?3, updated_at = ?4 WHERE id = ?5",
        params![title, description, quadrant, now, id],
    )?;
    Ok(())
}

pub fn complete_task(conn: &Connection, id: i64) -> Result<()> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "UPDATE tasks SET status = 'completed', completed_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

pub fn delete_task(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    Ok(())
}

pub fn update_task_quadrant(conn: &Connection, id: i64, quadrant: i32) -> Result<()> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "UPDATE tasks SET quadrant = ?1, updated_at = ?2 WHERE id = ?3",
        params![quadrant, now, id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn get_tasks(
    state: State<'_, Mutex<Connection>>,
    status: Option<String>,
) -> Result<Vec<Task>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    list_tasks(&conn, status).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_task(
    state: State<'_, Mutex<Connection>>,
    title: String,
    description: String,
    quadrant: i32,
) -> Result<Task, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    create_task(&conn, &title, &description, quadrant).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn edit_task(
    state: State<'_, Mutex<Connection>>,
    id: i64,
    title: String,
    description: String,
    quadrant: i32,
) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    update_task(&conn, id, &title, &description, quadrant).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn finish_task(state: State<'_, Mutex<Connection>>, id: i64) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    complete_task(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_task(state: State<'_, Mutex<Connection>>, id: i64) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    delete_task(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_task_quadrant(
    state: State<'_, Mutex<Connection>>,
    id: i64,
    quadrant: i32,
) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    update_task_quadrant(&conn, id, quadrant).map_err(|e| e.to_string())
}
