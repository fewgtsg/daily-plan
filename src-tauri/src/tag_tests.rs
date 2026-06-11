#[cfg(test)]
mod tests {
    use crate::db::{
        add_tag_to_entry, get_all_tags, get_entry_tags, get_task_tags, init_test_db,
        replace_entry_tags, replace_task_tags, search_entries_by_tag, search_tasks_by_tag,
    };

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

    #[test]
    fn replace_entry_tags_creates_tags_and_associations() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-01");
        replace_entry_tags(
            &conn,
            "2024-01-01",
            &vec!["work".to_string(), "ideas".to_string()],
        )
        .unwrap();
        let tags = get_entry_tags(&conn, "2024-01-01").unwrap();
        assert_eq!(tags.len(), 2);
        let names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"work".to_string()));
        assert!(names.contains(&"ideas".to_string()));
    }

    #[test]
    fn replace_entry_tags_deduplicates_and_preserves_order() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-02");
        replace_entry_tags(
            &conn,
            "2024-01-02",
            &vec![
                "alpha".to_string(),
                "beta".to_string(),
                "alpha".to_string(),
                "gamma".to_string(),
            ],
        )
        .unwrap();
        let tags = get_entry_tags(&conn, "2024-01-02").unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].name, "alpha");
        assert_eq!(tags[1].name, "beta");
        assert_eq!(tags[2].name, "gamma");
    }

    #[test]
    fn replace_entry_tags_handles_missing_entry() {
        let conn = init_test_db().unwrap();
        let result = replace_entry_tags(&conn, "2024-01-03", &vec!["work".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn replace_task_tags_works_for_tasks() {
        let conn = init_test_db().unwrap();
        let task_id = create_task(&conn, "task 1");
        replace_task_tags(
            &conn,
            task_id,
            &vec!["urgent".to_string(), "design".to_string()],
        )
        .unwrap();
        let tags = get_task_tags(&conn, task_id).unwrap();
        assert_eq!(tags.len(), 2);
        let names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"urgent".to_string()));
        assert!(names.contains(&"design".to_string()));
    }

    #[test]
    fn get_all_tags_returns_correct_usage_counts() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-04");
        create_entry(&conn, "2024-01-05");
        let task_id = create_task(&conn, "task 2");
        replace_entry_tags(&conn, "2024-01-04", &vec!["shared".to_string()]).unwrap();
        replace_entry_tags(&conn, "2024-01-05", &vec!["shared".to_string()]).unwrap();
        replace_task_tags(&conn, task_id, &vec!["shared".to_string()]).unwrap();
        let tags = get_all_tags(&conn).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].usage_count, 3);
    }

    #[test]
    fn get_entry_tags_returns_total_usage_count() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-06");
        create_entry(&conn, "2024-01-07");
        let task_id = create_task(&conn, "task 3");
        replace_entry_tags(&conn, "2024-01-06", &vec!["shared".to_string()]).unwrap();
        replace_entry_tags(&conn, "2024-01-07", &vec!["shared".to_string()]).unwrap();
        replace_task_tags(&conn, task_id, &vec!["shared".to_string()]).unwrap();
        let tags = get_entry_tags(&conn, "2024-01-06").unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].usage_count, 3);
    }

    #[test]
    fn search_entries_by_tag_works() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-08");
        replace_entry_tags(&conn, "2024-01-08", &vec!["searchable".to_string()]).unwrap();
        let entries = search_entries_by_tag(&conn, "searchable").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].date, "2024-01-08");
    }

    #[test]
    fn search_tasks_by_tag_works() {
        let conn = init_test_db().unwrap();
        let task_id = create_task(&conn, "search task");
        replace_task_tags(&conn, task_id, &vec!["searchable".to_string()]).unwrap();
        let tasks = search_tasks_by_tag(&conn, "searchable").unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "search task");
    }

    #[test]
    fn add_tag_to_entry_rejects_invalid_tags() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-09");
        assert!(add_tag_to_entry(&conn, "2024-01-09", "").is_err());
        assert!(add_tag_to_entry(&conn, "2024-01-09", "   ").is_err());
        assert!(add_tag_to_entry(&conn, "2024-01-09", &"a".repeat(51)).is_err());
        assert!(add_tag_to_entry(&conn, "2024-01-09", "123").is_err());
        assert!(add_tag_to_entry(&conn, "2024-01-09", "hello world").is_err());
        assert!(add_tag_to_entry(&conn, "2024-01-09", "hello!").is_err());
        assert!(add_tag_to_entry(&conn, "2024-01-09", "ok-tag_1").is_ok());
    }

    #[test]
    fn add_tag_to_entry_trims_whitespace() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-10");
        add_tag_to_entry(&conn, "2024-01-10", "  trimmed  ").unwrap();
        let tags = get_entry_tags(&conn, "2024-01-10").unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "trimmed");
    }

    #[test]
    fn get_task_tags_returns_task_tags() {
        let conn = init_test_db().unwrap();
        let task_id = create_task(&conn, "tagged task");
        replace_task_tags(
            &conn,
            task_id,
            &vec!["backend".to_string(), "rust".to_string()],
        )
        .unwrap();
        let tags = get_task_tags(&conn, task_id).unwrap();
        assert_eq!(tags.len(), 2);
        let names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"backend".to_string()));
        assert!(names.contains(&"rust".to_string()));
    }
}
