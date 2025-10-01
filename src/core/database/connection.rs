// SPDX-License-Identifier: GPL-3.0-only

//! Database connection management and configuration.
//!
//! This module handles creating database connections, ensuring directories exist,
//! and configuring `SQLite` pragmas for optimal performance.

use crate::core::database::{DatabaseError, Result};
use rusqlite::Connection;
use std::path::Path;

/// Creates a new database connection at the specified path.
///
/// # Errors
///
/// Returns an error if the connection cannot be established.
pub fn create_connection(path: &Path) -> Result<Connection> {
    Connection::open(path).map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))
}

/// Configures a database connection with optimal settings.
///
/// This function:
/// - Enables WAL (Write-Ahead Logging) mode for better concurrency
/// - Enables foreign key constraints
/// - Sets synchronous mode to NORMAL for better performance
///
/// # Errors
///
/// Returns an error if any pragma cannot be set.
pub fn configure_connection(conn: &Connection) -> Result<()> {
    // Enable WAL mode for better concurrency
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| DatabaseError::ConnectionFailed(format!("Failed to enable WAL mode: {e}")))?;

    // Enable foreign key constraints
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| {
            DatabaseError::ConnectionFailed(format!("Failed to enable foreign keys: {e}"))
        })?;

    // Set synchronous mode to NORMAL for better performance
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| {
            DatabaseError::ConnectionFailed(format!("Failed to set synchronous mode: {e}"))
        })?;

    Ok(())
}

/// Ensures the directory for the database file exists.
///
/// Creates all parent directories as needed.
///
/// # Errors
///
/// Returns an error if the directory cannot be created.
pub fn ensure_directory(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_connection_in_memory() {
        let conn = create_connection(Path::new(":memory:"));
        assert!(conn.is_ok());
    }

    #[test]
    fn test_create_connection_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = create_connection(&db_path);
        assert!(conn.is_ok());
        assert!(db_path.exists());
    }

    #[test]
    fn test_configure_connection_enables_wal() {
        // WAL mode doesn't work with in-memory databases, use a file
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        configure_connection(&conn).unwrap();

        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode.to_lowercase(), "wal");
    }

    #[test]
    fn test_configure_connection_enables_foreign_keys() {
        let conn = Connection::open_in_memory().unwrap();
        configure_connection(&conn).unwrap();

        let foreign_keys: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn test_configure_connection_sets_synchronous() {
        let conn = Connection::open_in_memory().unwrap();
        configure_connection(&conn).unwrap();

        let synchronous: i32 = conn
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .unwrap();
        assert_eq!(synchronous, 1); // NORMAL = 1
    }

    #[test]
    fn test_ensure_directory_creates_path() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("subdir/nested/db.sqlite");

        ensure_directory(&db_path).unwrap();
        assert!(db_path.parent().unwrap().exists());
    }

    #[test]
    fn test_ensure_directory_handles_existing() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("db.sqlite");

        // Should not error if directory already exists
        ensure_directory(&db_path).unwrap();
        ensure_directory(&db_path).unwrap();
    }

    #[test]
    fn test_ensure_directory_no_parent() {
        // Path with no parent (like ":memory:") should not error
        let result = ensure_directory(Path::new(":memory:"));
        assert!(result.is_ok());
    }
}
