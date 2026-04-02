//! Reverse compatibility: data written under v2 must remain readable by code that
//! only assumes the v1 column set (`key`, `value`).

use kvstore_rs::store;
use rusqlite::{params, Connection};

#[test]
fn test_v2_data_readable_by_v1_style_query() {
    let conn = Connection::open_in_memory().unwrap();

    store::init_v2(&conn).unwrap();
    store::set_value(&conn, "legacy-key", "stored-under-v2").unwrap();

    // Simulate a v1 reader: it only selects `value` and ignores `created_at`.
    let value: String = conn
        .query_row(
            "SELECT value FROM kv WHERE key = ?1",
            params!["legacy-key"],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(value, "stored-under-v2");
}
