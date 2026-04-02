use kvstore_rs::store;
use rusqlite::{params, Connection};

#[test]
fn test_v1_to_v2_compatibility() -> Result<(), store::KvError> {
    let conn = Connection::open_in_memory().unwrap();

    // Simulate a persisted v1 schema/data set.
    conn.execute(
        r#"
        CREATE TABLE kv (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO kv (key, value) VALUES (?1, ?2)",
        params!["greeting", "hello-v1"],
    )
    .unwrap();

    // New code should migrate and read old data.
    store::init_v2(&conn)?;
    let entry = store::get_entry(&conn, "greeting")?;

    assert_eq!(entry.value, "hello-v1");
    assert_eq!(entry.created_at, 0);
    Ok(())
}
