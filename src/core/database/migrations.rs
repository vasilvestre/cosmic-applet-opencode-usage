// SPDX-License-Identifier: GPL-3.0-only

//! Database schema migration system.
//!
//! This module handles versioning and applying database schema changes in a
//! controlled, incremental manner.

use crate::core::database::schema::{
    CREATE_DATE_INDEX, CREATE_SCHEMA_VERSION_TABLE, CREATE_USAGE_SNAPSHOTS_TABLE,
};
use crate::core::database::{DatabaseError, Result};
use rusqlite::Connection;

/// Represents a database schema migration.
#[derive(Debug)]
pub struct Migration {
    /// Version number of this migration.
    pub version: i32,
    /// Human-readable description of what this migration does.
    pub description: String,
    /// SQL statement(s) to execute for this migration.
    pub sql: String,
}

/// Returns the list of all available migrations in order.
#[must_use]
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "Initial schema - create usage_snapshots and schema_version tables"
                .to_string(),
            sql: format!(
                "{CREATE_SCHEMA_VERSION_TABLE};\n{CREATE_USAGE_SNAPSHOTS_TABLE};\n{CREATE_DATE_INDEX};"
            ),
        },
        Migration {
            version: 2,
            description: "Add UNIQUE constraint to date column to prevent duplicates".to_string(),
            sql: r#"
-- Create new table with UNIQUE constraint
CREATE TABLE usage_snapshots_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    reasoning_tokens INTEGER NOT NULL,
    cache_write_tokens INTEGER NOT NULL,
    cache_read_tokens INTEGER NOT NULL,
    total_cost REAL NOT NULL,
    interaction_count INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

-- Copy data, keeping only the most recent entry for each date
INSERT INTO usage_snapshots_new 
SELECT * FROM usage_snapshots
WHERE id IN (
    SELECT MAX(id) FROM usage_snapshots GROUP BY date
);

-- Drop old table
DROP TABLE usage_snapshots;

-- Rename new table
ALTER TABLE usage_snapshots_new RENAME TO usage_snapshots;

-- Recreate index
CREATE INDEX IF NOT EXISTS idx_usage_snapshots_date ON usage_snapshots(date);
"#
                .to_string(),
        },
    ]
}

/// Gets the current schema version from the database.
///
/// Returns 0 if the `schema_version` table doesn't exist (new database).
///
/// # Errors
///
/// Returns an error if the query fails for reasons other than the table not existing.
pub fn get_current_version(conn: &Connection) -> Result<i32> {
    // Check if schema_version table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| {
            DatabaseError::SchemaError(format!("Failed to check for schema_version table: {e}"))
        })?;

    if !table_exists {
        return Ok(0);
    }

    // Get the maximum version number
    let version: Option<i32> = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
            row.get(0)
        })
        .map_err(|e| DatabaseError::SchemaError(format!("Failed to get current version: {e}")))?;

    Ok(version.unwrap_or(0))
}

/// Applies all pending migrations to the database.
///
/// This function is idempotent - it's safe to call multiple times.
/// Only migrations with version numbers higher than the current version will be applied.
///
/// # Errors
///
/// Returns an error if any migration fails. The transaction will be rolled back.
pub fn apply_migrations(conn: &Connection) -> Result<()> {
    let current_version = get_current_version(conn)?;
    let migrations = get_migrations();

    for migration in migrations {
        if migration.version <= current_version {
            continue; // Skip already-applied migrations
        }

        apply_single_migration(conn, &migration)?;
    }

    Ok(())
}

/// Applies a single migration within a transaction.
///
/// # Errors
///
/// Returns an error if the migration fails. The transaction will be rolled back.
fn apply_single_migration(conn: &Connection, migration: &Migration) -> Result<()> {
    // Start transaction
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| DatabaseError::MigrationFailed(format!("Failed to start transaction: {e}")))?;

    // Execute the migration SQL
    let result = conn.execute_batch(&migration.sql);
    if let Err(e) = result {
        // Rollback on error
        let _ = conn.execute("ROLLBACK", []);
        return Err(DatabaseError::MigrationFailed(format!(
            "Failed to execute migration {}: {}",
            migration.version, e
        )));
    }

    // Record the migration
    let now = chrono::Utc::now().to_rfc3339();
    let result = conn.execute(
        "INSERT INTO schema_version (version, applied_at) VALUES (?1, ?2)",
        [&migration.version.to_string(), &now],
    );
    if let Err(e) = result {
        // Rollback on error
        let _ = conn.execute("ROLLBACK", []);
        return Err(DatabaseError::MigrationFailed(format!(
            "Failed to record migration {}: {}",
            migration.version, e
        )));
    }

    // Commit transaction
    conn.execute("COMMIT", []).map_err(|e| {
        DatabaseError::MigrationFailed(format!(
            "Failed to commit migration {}: {}",
            migration.version, e
        ))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_migrations_returns_list() {
        let migrations = get_migrations();
        assert!(!migrations.is_empty());
        assert_eq!(migrations[0].version, 1);
    }

    #[test]
    fn test_get_migrations_contains_initial_schema() {
        let migrations = get_migrations();
        assert!(migrations[0].sql.contains("usage_snapshots"));
        assert!(migrations[0].sql.contains("schema_version"));
    }

    #[test]
    fn test_get_current_version_on_new_db() {
        let conn = Connection::open_in_memory().unwrap();
        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, 0);
    }

    #[test]
    fn test_get_current_version_after_migration() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();

        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, 2); // Updated to expect version 2
    }

    #[test]
    fn test_apply_migrations_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();

        // Check tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();

        assert!(
            tables.contains(&"usage_snapshots".to_string()),
            "usage_snapshots table not found"
        );
        assert!(
            tables.contains(&"schema_version".to_string()),
            "schema_version table not found"
        );
    }

    #[test]
    fn test_apply_migrations_creates_index() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();

        // Check index exists
        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();

        assert!(
            indexes.contains(&"idx_usage_snapshots_date".to_string()),
            "Date index not found"
        );
    }

    #[test]
    fn test_apply_migrations_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();
        apply_migrations(&conn).unwrap(); // Should not error

        // Verify version is 2 (latest migration)
        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_apply_migrations_records_timestamp() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();

        // Check that applied_at was recorded
        let applied_at: String = conn
            .query_row(
                "SELECT applied_at FROM schema_version WHERE version = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        // Should be a valid ISO 8601 timestamp
        assert!(chrono::DateTime::parse_from_rfc3339(&applied_at).is_ok());
    }

    #[test]
    fn test_usage_snapshots_schema() {
        let conn = Connection::open_in_memory().unwrap();
        apply_migrations(&conn).unwrap();

        // Verify all columns exist by inserting a test row
        let result = conn.execute(
            "INSERT INTO usage_snapshots 
             (date, input_tokens, output_tokens, reasoning_tokens, 
              cache_write_tokens, cache_read_tokens, total_cost, 
              interaction_count, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                "2024-01-01",
                1000,
                2000,
                500,
                100,
                200,
                0.15,
                10,
                "2024-01-01T12:00:00Z"
            ],
        );

        assert!(result.is_ok());
    }
}
