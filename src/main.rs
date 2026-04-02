use kvstore_rs::store;
use rusqlite::Connection;

fn main() -> Result<(), store::KvError> {
    println!("Hello, world!");

    let conn = Connection::open("kvstore.db")?;

    store::init_v2(&conn)?;

    store::set_value(&conn, "greeting", "Hello from kvstore-rs!")?;
    let entry = store::get_entry(&conn, "greeting")?;

    println!(
        "kvstore initialized: value={}, created_at={}",
        entry.value, entry.created_at
    );

    Ok(())
}
