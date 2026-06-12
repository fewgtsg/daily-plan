#[cfg(test)]
mod tests {
    use crate::db::{get_entry_task_links, init_test_db, replace_entry_task_links};
    use crate::parser::ParsedTaskLink;

    fn create_entry(conn: &rusqlite::Connection, date: &str) -> i64 {
        conn.execute(
            "INSERT INTO entries (date, content, created_at, updated_at) VALUES (?1, '', datetime('now'), datetime('now'))",
            [date],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn create_task(conn: &rusqlite::Connection, title: &str) -> i64 {
        conn.execute(
            "INSERT INTO tasks (title, description, quadrant, created_at, updated_at) VALUES (?1, '', 1, datetime('now'), datetime('now'))",
            [title],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn link(raw_text: &str, position: usize) -> ParsedTaskLink {
        ParsedTaskLink {
            raw_text: raw_text.to_string(),
            position,
        }
    }

    #[test]
    fn replace_entry_task_links_creates_links() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-02-01");
        create_task(&conn, "Task A");
        replace_entry_task_links(&conn, "2024-02-01", &[link("Task A", 0)]).unwrap();
        let links = get_entry_task_links(&conn, "2024-02-01").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].raw_text, "Task A");
        assert!(links[0].task_id.is_some());
        assert_eq!(links[0].task_title, Some("Task A".to_string()));
    }

    #[test]
    fn replace_entry_task_links_marks_missing_tasks_as_invalid() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-02-02");
        replace_entry_task_links(&conn, "2024-02-02", &[link("Missing Task", 5)]).unwrap();
        let links = get_entry_task_links(&conn, "2024-02-02").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].raw_text, "Missing Task");
        assert!(links[0].task_id.is_none());
        assert_eq!(links[0].task_title, None);
    }

    #[test]
    fn replace_entry_task_links_removes_old_links() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-02-03");
        create_task(&conn, "Old Task");
        create_task(&conn, "New Task");
        replace_entry_task_links(&conn, "2024-02-03", &[link("Old Task", 0)]).unwrap();
        replace_entry_task_links(&conn, "2024-02-03", &[link("New Task", 10)]).unwrap();
        let links = get_entry_task_links(&conn, "2024-02-03").unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].raw_text, "New Task");
        assert_eq!(links[0].position, 10);
    }

    #[test]
    fn get_entry_task_links_returns_sorted_by_position() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-02-04");
        create_task(&conn, "Zebra");
        create_task(&conn, "Apple");
        replace_entry_task_links(
            &conn,
            "2024-02-04",
            &[link("Zebra", 20), link("Apple", 5), link("Mango", 15)],
        )
        .unwrap();
        let links = get_entry_task_links(&conn, "2024-02-04").unwrap();
        assert_eq!(links.len(), 3);
        let positions: Vec<i64> = links.iter().map(|l| l.position).collect();
        assert_eq!(positions, vec![5, 15, 20]);
        let raw_texts: Vec<String> = links.iter().map(|l| l.raw_text.clone()).collect();
        assert_eq!(raw_texts, vec!["Apple", "Mango", "Zebra"]);
    }

    #[test]
    fn replace_entry_task_links_handles_missing_entry() {
        let conn = init_test_db().unwrap();
        let result = replace_entry_task_links(&conn, "2024-02-05", &[link("Task", 0)]);
        assert!(result.is_ok());
        let links = get_entry_task_links(&conn, "2024-02-05").unwrap();
        assert!(links.is_empty());
    }
}
