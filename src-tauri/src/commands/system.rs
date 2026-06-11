use rusqlite::Connection;
use serde_json::json;
use std::fs;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn export_all_data(
    state: State<'_, Mutex<Connection>>,
    export_path: String,
) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    let mut stmt = conn
        .prepare("SELECT id, date, content, created_at, updated_at FROM entries")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "date": row.get::<_, String>(1)?,
                "content": row.get::<_, String>(2)?,
                "created_at": row.get::<_, String>(3)?,
                "updated_at": row.get::<_, String>(4)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        entries.push(row.map_err(|e| e.to_string())?);
    }

    let mut tasks = Vec::new();
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, quadrant, status, created_at, completed_at, updated_at FROM tasks",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "title": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "quadrant": row.get::<_, i32>(3)?,
                "status": row.get::<_, String>(4)?,
                "created_at": row.get::<_, String>(5)?,
                "completed_at": row.get::<_, Option<String>>(6)?,
                "updated_at": row.get::<_, String>(7)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    for row in rows {
        tasks.push(row.map_err(|e| e.to_string())?);
    }

    let data = json!({
        "entries": entries,
        "tasks": tasks,
        "exported_at": chrono::Local::now().to_rfc3339()
    });
    fs::write(
        &export_path,
        serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
