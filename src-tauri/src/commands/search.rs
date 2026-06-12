use crate::models::SearchResult;
use rusqlite::{Connection, Result};
use std::sync::Mutex;
use tauri::State;

pub fn search_all(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT e.id, e.date, e.content FROM entries_fts f JOIN entries e ON f.rowid = e.id WHERE f.content MATCH ?1 ORDER BY rank"
    )?;
    let rows = stmt.query_map([query], |row| {
        Ok(SearchResult {
            result_type: "entry".to_string(),
            id: row.get(0)?,
            date: row.get(1)?,
            content: row.get(2)?,
        })
    })?;
    for row in rows {
        results.push(row?);
    }
    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, t.description FROM tasks_fts f JOIN tasks t ON f.rowid = t.id WHERE f.tasks_fts MATCH ?1 ORDER BY rank"
    )?;
    let rows = stmt.query_map([query], |row| {
        let title: String = row.get(1)?;
        let desc: String = row.get(2)?;
        Ok(SearchResult {
            result_type: "task".to_string(),
            id: row.get(0)?,
            date: None,
            content: format!("{} {}", title, desc),
        })
    })?;
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

#[tauri::command]
pub fn search(
    state: State<'_, Mutex<Connection>>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let conn = state.lock().map_err(|e| e.to_string())?;
    search_all(&conn, &query).map_err(|e| e.to_string())
}
