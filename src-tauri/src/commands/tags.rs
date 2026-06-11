use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;
use rusqlite::Connection;

use crate::models::TagDto;
use crate::parser::extract_tags;

#[tauri::command]
pub fn sync_entry_tags(state: State<'_, Mutex<Connection>>, date: String, content: String) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let parsed = extract_tags(&content);
    crate::db::replace_entry_tags(&conn, &date, &parsed.names).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag_to_entry(state: State<'_, Mutex<Connection>>, date: String, tag_name: String) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::db::add_tag_to_entry(&conn, &date, &tag_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_entry_tags(state: State<'_, Mutex<Connection>>, date: String) -> Result<Vec<TagDto>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::db::get_entry_tags(&conn, &date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn sync_task_tags(state: State<'_, Mutex<Connection>>, task_id: i64, content: String) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let parsed = extract_tags(&content);
    crate::db::replace_task_tags(&conn, task_id, &parsed.names).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_tags(state: State<'_, Mutex<Connection>>) -> Result<Vec<TagDto>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::db::get_all_tags(&conn).map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize)]
pub struct TagSearchResult {
    pub entries: Vec<crate::models::Entry>,
    pub tasks: Vec<crate::models::Task>,
}

#[tauri::command]
pub fn search_by_tag(state: State<'_, Mutex<Connection>>, tag_name: String) -> Result<TagSearchResult, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    let entries = crate::db::search_entries_by_tag(&conn, &tag_name).map_err(|e| e.to_string())?;
    let tasks = crate::db::search_tasks_by_tag(&conn, &tag_name).map_err(|e| e.to_string())?;
    Ok(TagSearchResult { entries, tasks })
}
