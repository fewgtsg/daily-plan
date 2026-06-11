use rusqlite::{Connection, Result};
use std::fs;
use tauri::Manager;

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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            display_name TEXT,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entry_tags (
            entry_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (entry_id, tag_id),
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_tags (
            task_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (task_id, tag_id),
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entry_task_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL,
            task_id INTEGER,
            raw_text TEXT NOT NULL,
            position INTEGER,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
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
