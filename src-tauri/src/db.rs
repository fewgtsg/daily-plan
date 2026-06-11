use rusqlite::{Connection, OptionalExtension, Result, Transaction};
use std::fs;
use tauri::Manager;

use crate::models::TagDto;

pub fn init_app_db(app_handle: &tauri::AppHandle) -> Result<Connection> {
    let app_dir = app_handle.path().app_local_data_dir().expect("Failed to get app data dir");
    fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
    let db_path = app_dir.join("app.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    create_tables(&conn)?;
    create_v2_tables(&conn)?;
    Ok(conn)
}

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT UNIQUE NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            quadrant INTEGER NOT NULL CHECK(quadrant BETWEEN 1 AND 4),
            status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'completed', 'archived')),
            created_at TEXT NOT NULL,
            completed_at TEXT,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS entry_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
            task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            linked_at TEXT NOT NULL,
            UNIQUE(entry_id, task_id)
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
            content,
            content='entries',
            content_rowid='id'
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
            title, description,
            content='tasks',
            content_rowid='id'
        );

        CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(rowid, content) VALUES (new.id, new.content);
        END;
        CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, content) VALUES ('delete', old.id, old.content);
        END;
        CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, content) VALUES ('delete', old.id, old.content);
            INSERT INTO entries_fts(rowid, content) VALUES (new.id, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS tasks_ai AFTER INSERT ON tasks BEGIN
            INSERT INTO tasks_fts(rowid, title, description) VALUES (new.id, new.title, new.description);
        END;
        CREATE TRIGGER IF NOT EXISTS tasks_ad AFTER DELETE ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, description) VALUES ('delete', old.id, old.title, old.description);
        END;
        CREATE TRIGGER IF NOT EXISTS tasks_au AFTER UPDATE ON tasks BEGIN
            INSERT INTO tasks_fts(tasks_fts, rowid, title, description) VALUES ('delete', old.id, old.title, old.description);
            INSERT INTO tasks_fts(rowid, title, description) VALUES (new.id, new.title, new.description);
        END;"
    )?;
    Ok(())
}

fn create_v2_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            display_name TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS entry_tags (
            entry_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (entry_id, tag_id),
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS task_tags (
            task_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (task_id, tag_id),
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS entry_task_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL,
            task_id INTEGER,
            raw_text TEXT NOT NULL,
            position INTEGER,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL
        );

        CREATE INDEX IF NOT EXISTS idx_entry_task_links_entry_id ON entry_task_links(entry_id);
        CREATE INDEX IF NOT EXISTS idx_entry_task_links_task_id ON entry_task_links(task_id);

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );"
    )?;
    Ok(())
}

fn dedup_tags_preserve_order(tags: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for tag in tags {
        let normalized = tag.to_lowercase();
        if seen.insert(normalized) {
            result.push(tag.clone());
        }
    }
    result
}

fn ensure_tag(tx: &Transaction, name: &str, display_name: &str) -> Result<i64> {
    tx.execute(
        "INSERT INTO tags (name, display_name, created_at) VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(name) DO UPDATE SET display_name = COALESCE(tags.display_name, excluded.display_name)",
        [name, display_name],
    )?;
    let tag_id: i64 = tx.query_row(
        "SELECT id FROM tags WHERE name = ?1",
        [name],
        |row| row.get(0),
    )?;
    Ok(tag_id)
}

pub fn replace_entry_tags(conn: &Connection, date: &str, tags: &[String]) -> Result<()> {
    let unique_tags = dedup_tags_preserve_order(tags);

    let entry_id: Option<i64> = conn.query_row(
        "SELECT id FROM entries WHERE date = ?1",
        [date],
        |row| row.get(0),
    ).optional()?;
    let entry_id = match entry_id {
        Some(id) => id,
        None => return Ok(()),
    };

    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM entry_tags WHERE entry_id = ?1", [entry_id])?;
    for name in &unique_tags {
        let tag_id = ensure_tag(&tx, name.as_str(), name.as_str())?;
        tx.execute(
            "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
            [entry_id, tag_id],
        )?;
    }
    tx.commit()
}

pub fn replace_task_tags(conn: &Connection, task_id: i64, tags: &[String]) -> Result<()> {
    let unique_tags = dedup_tags_preserve_order(tags);
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM task_tags WHERE task_id = ?1", [task_id])?;
    for name in &unique_tags {
        let tag_id = ensure_tag(&tx, name.as_str(), name.as_str())?;
        tx.execute(
            "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)",
            [task_id, tag_id],
        )?;
    }
    tx.commit()
}

fn validate_tag_name(tag_name: &str) -> Result<String> {
    let trimmed = tag_name.trim();
    if trimmed.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName(
            "Tag name cannot be empty".to_string(),
        ));
    }
    if trimmed.chars().count() > 50 {
        return Err(rusqlite::Error::InvalidParameterName(
            "Tag name must be 50 characters or fewer".to_string(),
        ));
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(rusqlite::Error::InvalidParameterName(
            "Tag name cannot be purely numeric".to_string(),
        ));
    }
    if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(rusqlite::Error::InvalidParameterName(
            "Tag name can only contain letters, numbers, underscores, and hyphens".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

pub fn add_tag_to_entry(conn: &Connection, date: &str, tag_name: &str) -> Result<()> {
    let trimmed = validate_tag_name(tag_name)?;
    let normalized = trimmed.to_lowercase();

    let entry_id: i64 = conn.query_row(
        "SELECT id FROM entries WHERE date = ?1",
        [date],
        |row| row.get(0),
    )?;

    let tx = conn.unchecked_transaction()?;
    let tag_id = ensure_tag(&tx, &normalized, &trimmed)?;
    tx.execute(
        "INSERT OR IGNORE INTO entry_tags (entry_id, tag_id) VALUES (?1, ?2)",
        [entry_id, tag_id],
    )?;
    tx.commit()
}

pub fn get_entry_tags(conn: &Connection, date: &str) -> Result<Vec<TagDto>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.display_name,
                (SELECT COUNT(*) FROM entry_tags et WHERE et.tag_id = t.id) +
                (SELECT COUNT(*) FROM task_tags tt WHERE tt.tag_id = t.id) AS usage_count
         FROM tags t
         JOIN entry_tags et ON et.tag_id = t.id
         JOIN entries e ON e.id = et.entry_id
         WHERE e.date = ?1
         GROUP BY t.id"
    )?;
    let rows = stmt.query_map([date], |row| {
        Ok(TagDto {
            id: row.get(0)?,
            name: row.get(1)?,
            display_name: row.get(2)?,
            usage_count: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn get_task_tags(conn: &Connection, task_id: i64) -> Result<Vec<TagDto>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.display_name,
                (SELECT COUNT(*) FROM entry_tags et WHERE et.tag_id = t.id) +
                (SELECT COUNT(*) FROM task_tags tt WHERE tt.tag_id = t.id) AS usage_count
         FROM tags t
         JOIN task_tags tt ON tt.tag_id = t.id
         WHERE tt.task_id = ?1
         ORDER BY t.name ASC"
    )?;
    let rows = stmt.query_map([task_id], |row| {
        Ok(TagDto {
            id: row.get(0)?,
            name: row.get(1)?,
            display_name: row.get(2)?,
            usage_count: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn get_all_tags(conn: &Connection) -> Result<Vec<TagDto>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.display_name,
                (SELECT COUNT(*) FROM entry_tags et WHERE et.tag_id = t.id) +
                (SELECT COUNT(*) FROM task_tags tt WHERE tt.tag_id = t.id) AS usage_count
         FROM tags t
         ORDER BY usage_count DESC, t.name ASC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(TagDto {
            id: row.get(0)?,
            name: row.get(1)?,
            display_name: row.get(2)?,
            usage_count: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn search_entries_by_tag(conn: &Connection, tag_name: &str) -> Result<Vec<crate::models::Entry>> {
    // Query entries that have the given tag
    let mut stmt = conn.prepare(
        "SELECT e.id, e.date, e.content, e.created_at, e.updated_at
         FROM entries e
         JOIN entry_tags et ON et.entry_id = e.id
         JOIN tags t ON t.id = et.tag_id
         WHERE t.name = ?1
         ORDER BY e.date DESC"
    )?;
    let rows = stmt.query_map([tag_name.to_lowercase()], |row| {
        Ok(crate::models::Entry {
            id: row.get(0)?,
            date: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn search_tasks_by_tag(conn: &Connection, tag_name: &str) -> Result<Vec<crate::models::Task>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, t.description, t.quadrant, t.status, t.created_at, t.completed_at, t.updated_at
         FROM tasks t
         JOIN task_tags tt ON tt.task_id = t.id
         JOIN tags tg ON tg.id = tt.tag_id
         WHERE tg.name = ?1
         ORDER BY t.updated_at DESC"
    )?;
    let rows = stmt.query_map([tag_name.to_lowercase()], |row| {
        Ok(crate::models::Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            quadrant: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
            completed_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn backup_db(app_handle: &tauri::AppHandle) -> std::io::Result<()> {
    let app_dir = app_handle.path().app_local_data_dir().expect("Failed to get app data dir");
    let db_path = app_dir.join("app.db");
    let backup_dir = app_dir.join("backups");
    std::fs::create_dir_all(&backup_dir)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let backup_path = backup_dir.join(format!("app-{}.db", today));
    if !backup_path.exists() {
        std::fs::copy(&db_path, &backup_path)?;
        if let Ok(entries) = std::fs::read_dir(&backup_dir) {
            let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            files.sort_by_key(|e| std::cmp::Reverse(e.path()));
            for entry in files.iter().skip(7) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
pub fn init_test_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    create_tables(&conn)?;
    create_v2_tables(&conn)?;
    Ok(conn)
}
