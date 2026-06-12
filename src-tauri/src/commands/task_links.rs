use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

use crate::models::TaskLinkDto;

#[tauri::command]
pub fn sync_entry_task_links(
    state: State<'_, Mutex<Connection>>,
    date: String,
    content: String,
) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let links = crate::parser::extract_task_links(&content);
    crate::db::replace_entry_task_links(&conn, &date, &links).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_entry_task_links(
    state: State<'_, Mutex<Connection>>,
    date: String,
) -> Result<Vec<TaskLinkDto>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::db::get_entry_task_links(&conn, &date).map_err(|e| e.to_string())
}
