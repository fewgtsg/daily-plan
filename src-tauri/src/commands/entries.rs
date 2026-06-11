use crate::models::Entry;
use rusqlite::{params, Connection, Result};

pub fn get_entry_by_date(conn: &Connection, date: &str) -> Result<Option<Entry>> {
    let mut stmt = conn
        .prepare("SELECT id, date, content, created_at, updated_at FROM entries WHERE date = ?1")?;
    let mut rows = stmt.query_map([date], |row| {
        Ok(Entry {
            id: row.get(0)?,
            date: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn create_or_update_entry(conn: &Connection, date: &str, content: &str) -> Result<Entry> {
    let now = chrono::Local::now().to_rfc3339();
    let existing = get_entry_by_date(conn, date)?;
    if let Some(entry) = existing {
        conn.execute(
            "UPDATE entries SET content = ?1, updated_at = ?2 WHERE id = ?3",
            params![content, now, entry.id],
        )?;
        Ok(Entry {
            id: entry.id,
            date: date.to_string(),
            content: content.to_string(),
            created_at: entry.created_at,
            updated_at: now,
        })
    } else {
        conn.execute(
            "INSERT INTO entries (date, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![date, content, now, now],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Entry {
            id,
            date: date.to_string(),
            content: content.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }
}

pub fn list_entry_dates(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT date FROM entries ORDER BY date DESC")?;
    let dates = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>>>()?;
    Ok(dates)
}

use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_entry(
    state: State<'_, Mutex<Connection>>,
    date: String,
) -> Result<Option<Entry>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    get_entry_by_date(&conn, &date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_entry(
    state: State<'_, Mutex<Connection>>,
    date: String,
    content: String,
) -> Result<Entry, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    create_or_update_entry(&conn, &date, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_entry_dates(state: State<'_, Mutex<Connection>>) -> Result<Vec<String>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    list_entry_dates(&conn).map_err(|e| e.to_string())
}
