use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_app_setting(
    state: State<'_, Mutex<Connection>>,
    key: String,
) -> Result<Option<String>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::db::get_setting(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_app_setting(
    state: State<'_, Mutex<Connection>>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    crate::db::set_setting(&conn, &key, &value).map_err(|e| e.to_string())
}
