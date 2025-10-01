# Database Integration

This document describes the database integration feature for persistent storage of OpenCode usage metrics.

## Overview

The database module provides SQLite-based storage for tracking usage metrics over time. It includes:

- **Automatic initialization** - Database and schema created on first use
- **Schema migrations** - Versioned schema updates
- **Thread-safe access** - Safe concurrent operations
- **WAL mode** - Write-Ahead Logging for better performance

## Architecture

```
src/core/database/
├── mod.rs          # DatabaseManager and public API
├── schema.rs       # SQL table definitions
├── connection.rs   # Connection management
└── migrations.rs   # Schema versioning
```

## Database Location

Default path: `~/.local/share/cosmic-applet-opencode-usage/usage.db`

## Schema

### `usage_snapshots` Table

Stores daily usage metrics:

| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Primary key (auto-increment) |
| date | TEXT | ISO 8601 date |
| input_tokens | INTEGER | Total input tokens used |
| output_tokens | INTEGER | Total output tokens generated |
| reasoning_tokens | INTEGER | Total reasoning tokens used |
| cache_write_tokens | INTEGER | Total cache write tokens |
| cache_read_tokens | INTEGER | Total cache read tokens |
| total_cost | REAL | Total cost in dollars |
| interaction_count | INTEGER | Number of interactions |
| created_at | TEXT | ISO 8601 timestamp |

### `schema_version` Table

Tracks applied migrations:

| Column | Type | Description |
|--------|------|-------------|
| version | INTEGER | Migration version number |
| applied_at | TEXT | ISO 8601 timestamp |

## Usage

### Initialize Database

```rust
use cosmic_applet_opencode_usage::core::database::DatabaseManager;

// Use default path
let manager = DatabaseManager::new()?;

// Or specify custom path (useful for testing)
let manager = DatabaseManager::new_with_path(&custom_path)?;
```

### Insert Data

```rust
let conn = manager.get_connection();
conn.execute(
    "INSERT INTO usage_snapshots 
     (date, input_tokens, output_tokens, reasoning_tokens, 
      cache_write_tokens, cache_read_tokens, total_cost, 
      interaction_count, created_at) 
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    rusqlite::params![
        "2024-01-15",
        15420,
        8234,
        2145,
        1230,
        567,
        0.387,
        42,
        "2024-01-15T18:30:00Z"
    ],
)?;
```

### Query Data

```rust
let conn = manager.get_connection();
let mut stmt = conn.prepare(
    "SELECT date, total_cost FROM usage_snapshots 
     WHERE date >= ?1 ORDER BY date"
)?;

let rows = stmt.query_map(["2024-01-01"], |row| {
    Ok((
        row.get::<_, String>(0)?,
        row.get::<_, f64>(1)?,
    ))
})?;

for row in rows {
    let (date, cost) = row?;
    println!("{}: ${:.2}", date, cost);
}
```

## Thread Safety

The `DatabaseManager` uses `Mutex<Connection>` to ensure thread-safe access:

```rust
use std::sync::Arc;
use std::thread;

let manager = Arc::new(DatabaseManager::new()?);

// Spawn multiple threads
let handles: Vec<_> = (0..5).map(|_| {
    let manager = Arc::clone(&manager);
    thread::spawn(move || {
        let conn = manager.get_connection();
        // Perform database operations
    })
}).collect();

for handle in handles {
    handle.join().unwrap();
}
```

## Migrations

The migration system ensures the database schema stays up-to-date:

- **Automatic application** - Migrations run on initialization
- **Idempotent** - Safe to run multiple times
- **Transactional** - Rollback on failure
- **Versioned** - Track which migrations have been applied

### Adding a New Migration

1. Add to `migrations.rs`:
```rust
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "Initial schema".to_string(),
            sql: "...".to_string(),
        },
        Migration {
            version: 2,
            description: "Add new column".to_string(),
            sql: "ALTER TABLE usage_snapshots ADD COLUMN new_field TEXT".to_string(),
        },
    ]
}
```

2. Migration will be applied automatically on next initialization

## Performance

- **WAL Mode**: Enabled for better concurrent access
- **Indexed**: Date column indexed for efficient queries
- **Synchronous**: Set to NORMAL for good balance of speed and safety
- **Connection reuse**: Single connection with mutex (sufficient for applet use case)

## Error Handling

All database operations return `Result<T, DatabaseError>`:

```rust
pub enum DatabaseError {
    ConnectionFailed(String),
    MigrationFailed(String),
    SchemaError(String),
    IoError(std::io::Error),
    SqlError(rusqlite::Error),
}
```

Example error handling:

```rust
match DatabaseManager::new() {
    Ok(manager) => { /* use manager */ },
    Err(DatabaseError::ConnectionFailed(msg)) => {
        eprintln!("Failed to connect: {}", msg);
    },
    Err(e) => {
        eprintln!("Database error: {}", e);
    },
}
```

## Testing

Run database tests:

```bash
# All database tests
cargo test database

# Integration tests only
cargo test --test database_integration

# Specific test
cargo test test_database_manager_new_with_path
```

## Example

See `examples/database_usage.rs` for a complete working example:

```bash
cargo run --example database_usage
```

## Future Enhancements

Potential additions to the database module:

1. **Query API** - High-level methods for common queries
2. **Aggregation** - Time-based summaries (daily, weekly, monthly)
3. **Data retention** - Automatic cleanup of old records
4. **Export** - Export data to JSON/CSV
5. **Backup** - Automated backup functionality

## Dependencies

- `rusqlite` (v0.32) - SQLite bindings with bundled library
- `chrono` - Date/time handling for timestamps

## License

GPL-3.0-only
