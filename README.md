# kvstore-rs

SQLite-backed key/value storage in Rust: **v1->v2 migration**, forward and reverse compatibility tests, and CI on MSRV plus stable.

## What it does

- **`init_v2`** - ensures table `kv` with `key`, `value`, `created_at`; migrates legacy v1 tables (adds `created_at` default `0`).
- **`set_value` / `get_entry`** - upsert and read; missing keys return `KvError::NotFound`.

## Run

Build and run the default binary:

```bash
cargo run
```

Creates `kvstore.db` in the working directory, prints `Hello, world!`, then writes and reads a sample key.

## Example

Library usage:

```rust
use kvstore_rs::store;
use rusqlite::Connection;

fn main() -> Result<(), store::KvError> {
    let conn = Connection::open("kvstore.db")?;
    store::init_v2(&conn)?;
    store::set_value(&conn, "key", "value")?;
    let entry = store::get_entry(&conn, "key")?;
    println!("{} (created_at={})", entry.value, entry.created_at);
    Ok(())
}
```

## Schema and migration

```mermaid
flowchart LR
  subgraph v1["v1 (legacy)"]
    A["key TEXT PK"]
    B["value TEXT"]
  end
  subgraph v2["v2 (current)"]
    C["key TEXT PK"]
    D["value TEXT"]
    E["created_at INTEGER DEFAULT 0"]
  end
  v1 -->|"ALTER TABLE ... ADD COLUMN"| v2

  style v1 fill:#dbeafe,stroke:#1d4ed8,stroke-width:2px,color:#1e3a8a
  style v2 fill:#dcfce7,stroke:#15803d,stroke-width:2px,color:#14532d
  style A fill:#eff6ff,stroke:#3b82f6
  style B fill:#eff6ff,stroke:#3b82f6
  style C fill:#f0fdf4,stroke:#22c55e
  style D fill:#f0fdf4,stroke:#22c55e
  style E fill:#f0fdf4,stroke:#22c55e
```

Legacy readers can keep using `SELECT value FROM kv WHERE key = ?`; new code should call `init_v2` first so existing databases get the new column.

## Project layout

| Path | Role |
|------|------|
| `src/lib.rs` | Exposes `store`. |
| `src/store.rs` | Schema, migration, API, unit tests. |
| `src/main.rs` | Sample binary. |
| `tests/compatibility.rs` | v1 data -> `init_v2` -> read. |
| `tests/reverse_compatibility.rs` | v2 write -> v1-style SQL read. |

## Tests

- Unit: v2 roundtrip; missing key -> `NotFound`.
- Integration: forward (v1->v2) and reverse (v2 data readable with v1-shaped queries).

```bash
cargo test --all-targets --all-features
```

## Minimum supported Rust version

**1.82** (`Cargo.toml` `rust-version`), aligned with CI and dependency/toolchain constraints (not an arbitrary pin).

## Continuous integration

On every **push** and **pull request**, a matrix runs **1.82.0** and **stable**:

```mermaid
flowchart TD
  M["Matrix: 1.82.0, stable"] --> C["cargo check --all-targets --all-features"]
  M --> F["cargo fmt --check"]
  M --> L["cargo clippy -D warnings"]
  M --> T["cargo test --all-targets --all-features"]

  style M fill:#fef9c3,stroke:#ca8a04,stroke-width:2px,color:#713f12
  style C fill:#e0e7ff,stroke:#4f46e5,stroke-width:2px,color:#312e81
  style F fill:#fce7f3,stroke:#db2777,stroke-width:2px,color:#831843
  style L fill:#ffedd5,stroke:#ea580c,stroke-width:2px,color:#7c2d12
  style T fill:#d1fae5,stroke:#059669,stroke-width:2px,color:#064e3b
```

**Signed commits** (separate workflow): on **pull requests** and **pushes to `main`**, verifies non-bot commits via the GitHub API (`commit.verification`); failures list short SHAs and reasons.

## License

Licensed under the **MIT License**. See [`LICENSE`](LICENSE). SPDX: `MIT`.
