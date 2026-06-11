#[cfg(test)]
mod tests {
    use crate::db::{
        add_tag_to_entry, get_all_tags, get_entry_tags, get_task_tags, init_test_db,
        replace_entry_tags, replace_task_tags, search_entries_by_tag, search_tasks_by_tag,
    };
    use crate::parser::ParsedTag;

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

    fn tag(original: &str) -> ParsedTag {
        ParsedTag {
            original: original.to_string(),
            normalized: original.to_lowercase(),
        }
    }

    #[test]
    fn replace_entry_tags_creates_tags_and_associations() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-01");
        replace_entry_tags(&conn, "2024-01-01", &[tag("work"), tag("ideas")]).unwrap();
        let tags = get_entry_tags(&conn, "2024-01-01").unwrap();
        assert_eq!(tags.len(), 2);
        let names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"work".to_string()));
        assert!(names.contains(&"ideas".to_string()));
    }

    #[test]
    fn replace_entry_tags_deduplicates_tags() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-02");
        replace_entry_tags(
            &conn,
            "2024-01-02",
            &[
                tag("beta"),
                tag("alpha"),
                tag("beta"),
                tag("gamma"),
            ],
        )
        .unwrap();
        let tags = get_entry_tags(&conn, "2024-01-02").unwrap();
        let names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn get_entry_tags_returns_sorted_by_name() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-12");
        replace_entry_tags(
            &conn,
            "2024-01-12",
            &[tag("zebra"), tag("apple"), tag("mango")],
        )
        .unwrap();
        let tags = get_entry_tags(&conn, "2024-01-12").unwrap();
        let names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert_eq!(names, vec!["apple", "mango", "zebra"]);
    }

    #[test]
    fn replace_entry_tags_handles_missing_entry() {
        let conn = init_test_db().unwrap();
        let result = replace_entry_tags(&conn, "2024-01-03", &[tag("work")]);
        assert!(result.is_ok());
    }

    #[test]
    fn replace_task_tags_handles_missing_task() {
        let conn = init_test_db().unwrap();
        let result = replace_task_tags(&conn, 999, &[tag("work")]);
        assert!(result.is_ok());
    }

    #[test]
    fn replace_task_tags_works_for_tasks() {
        let conn = init_test_db().unwrap();
        let task_id = create_task(&conn, "task 1");
        replace_task_tags(&conn, task_id, &[tag("urgent"), tag("design")]).unwrap();
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
        replace_entry_tags(&conn, "2024-01-04", &[tag("shared")]).unwrap();
        replace_entry_tags(&conn, "2024-01-05", &[tag("shared")]).unwrap();
        replace_task_tags(&conn, task_id, &[tag("shared")]).unwrap();
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
        replace_entry_tags(&conn, "2024-01-06", &[tag("shared")]).unwrap();
        replace_entry_tags(&conn, "2024-01-07", &[tag("shared")]).unwrap();
        replace_task_tags(&conn, task_id, &[tag("shared")]).unwrap();
        let tags = get_entry_tags(&conn, "2024-01-06").unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].usage_count, 3);
    }

    #[test]
    fn search_entries_by_tag_works() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-08");
        replace_entry_tags(&conn, "2024-01-08", &[tag("searchable")]).unwrap();
        let entries = search_entries_by_tag(&conn, "searchable").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].date, "2024-01-08");
    }

    #[test]
    fn search_tasks_by_tag_works() {
        let conn = init_test_db().unwrap();
        let task_id = create_task(&conn, "search task");
        replace_task_tags(&conn, task_id, &[tag("searchable")]).unwrap();
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
    fn replace_entry_tags_rejects_invalid_tags() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-11");
        assert!(replace_entry_tags(&conn, "2024-01-11", &[tag("")]).is_err());
        assert!(replace_entry_tags(&conn, "2024-01-11", &[tag("   ")]).is_err());
        assert!(replace_entry_tags(&conn, "2024-01-11", &[tag(&"a".repeat(51))]).is_err());
        assert!(replace_entry_tags(&conn, "2024-01-11", &[tag("123")]).is_err());
        assert!(replace_entry_tags(&conn, "2024-01-11", &[tag("hello world")]).is_err());
        assert!(replace_entry_tags(&conn, "2024-01-11", &[tag("hello!")]).is_err());
        assert!(replace_entry_tags(&conn, "2024-01-11", &[tag("ok-tag_1")]).is_ok());
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
        replace_task_tags(&conn, task_id, &[tag("backend"), tag("rust")]).unwrap();
        let tags = get_task_tags(&conn, task_id).unwrap();
        assert_eq!(tags.len(), 2);
        let names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"backend".to_string()));
        assert!(names.contains(&"rust".to_string()));
    }

    #[test]
    fn replace_entry_tags_preserves_original_casing_for_display_name() {
        let conn = init_test_db().unwrap();
        create_entry(&conn, "2024-01-13");
        replace_entry_tags(&conn, "2024-01-13", &[tag("Work"), tag("WORK")]).unwrap();
        let tags = get_entry_tags(&conn, "2024-01-13").unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "work");
        assert_eq!(tags[0].display_name, Some("Work".to_string()));
    }
}
