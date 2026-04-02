//! SQLite-backed key/value store with backward-compatible persistence.

use core::fmt;
use rusqlite::{params, Connection, OptionalExtension};

/// Error type for this crate.
#[derive(Debug)]
pub enum KvError {
    /// A SQLite (rusqlite) error.
    Db(rusqlite::Error),
    /// Entry was not found for the requested key.
    NotFound,
}

impl fmt::Display for KvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KvError::Db(e) => e.fmt(f),
            KvError::NotFound => write!(f, "key not found"),
        }
    }
}

impl std::error::Error for KvError {}

impl From<rusqlite::Error> for KvError {
    fn from(e: rusqlite::Error) -> Self {
        KvError::Db(e)
    }
}

/// Entry returned by [`get_entry`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub value: String,
    pub created_at: i64,
}

/// Result alias used throughout this module.
type Result<T> = core::result::Result<T, KvError>;

fn has_created_at_column(conn: &Connection) -> Result<bool> {
    let mut stmt = conn.prepare("PRAGMA table_info(kv)")?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        // PRAGMA table_info returns: cid, name, type, notnull, dflt_value, pk
        let name: String = row.get(1)?;
        if name == "created_at" {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Initialize the v2 database schema.
///
/// v2 stores `key` and `value`, plus `created_at` (defaulting to `0`).
pub fn init_v2(conn: &Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS kv (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT 0
        )
        "#,
        [],
    )?;

    // Ensure schema is upgraded to v2 before use.
    let has_column = has_created_at_column(conn)?;
    if !has_column {
        conn.execute(
            "ALTER TABLE kv ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    Ok(())
}

/// Set a value for `key` and update the row if it already exists.
///
/// This assumes the caller has run [`init_v2`], which ensures the `created_at`
/// column exists (including one-time v1->v2 migration).
pub fn set_value(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO kv (key, value, created_at)
        VALUES (?1, ?2, 0)
        ON CONFLICT(key) DO UPDATE SET
            value = excluded.value,
            created_at = excluded.created_at
        "#,
        params![key, value],
    )?;

    Ok(())
}

/// Get an entry for `key`.
///
/// This assumes the caller has run [`init_v2`]. For rows upgraded from v1,
/// `created_at` is defaulted to `0`.
pub fn get_entry(conn: &Connection, key: &str) -> Result<Entry> {
    let maybe = conn
        .query_row(
            "SELECT value, created_at FROM kv WHERE key = ?1",
            params![key],
            |row| {
                Ok(Entry {
                    value: row.get(0)?,
                    created_at: row.get(1)?,
                })
            },
        )
        .optional()?;

    maybe.ok_or(KvError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2_roundtrip() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        init_v2(&conn)?;

        set_value(&conn, "k", "v")?;
        let entry = get_entry(&conn, "k")?;

        assert_eq!(entry.value, "v");
        assert_eq!(entry.created_at, 0);
        Ok(())
    }

    #[test]
    fn test_missing_key_returns_error() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        init_v2(&conn)?;

        let err = get_entry(&conn, "missing").expect_err("missing key should error");
        assert!(matches!(err, KvError::NotFound));
        Ok(())
    }
}
