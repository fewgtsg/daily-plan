#[cfg(test)]
mod tests {
    use crate::db::{get_setting, init_test_db, set_setting};

    #[test]
    fn set_and_get_setting() {
        let conn = init_test_db().unwrap();
        set_setting(&conn, "theme", "dark").unwrap();
        let value = get_setting(&conn, "theme").unwrap();
        assert_eq!(value, Some("dark".to_string()));
    }

    #[test]
    fn get_missing_setting_returns_none() {
        let conn = init_test_db().unwrap();
        let value = get_setting(&conn, "missing_key").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn set_overwrites_existing_value() {
        let conn = init_test_db().unwrap();
        set_setting(&conn, "language", "en").unwrap();
        set_setting(&conn, "language", "zh").unwrap();
        let value = get_setting(&conn, "language").unwrap();
        assert_eq!(value, Some("zh".to_string()));
    }
}
