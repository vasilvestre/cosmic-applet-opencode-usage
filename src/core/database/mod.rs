// SPDX-License-Identifier: GPL-3.0-only

//! Database integration for persistent storage of `OpenCode` usage metrics.
//!
//! This module provides SQLite-based storage for tracking usage metrics over time,
//! enabling historical analysis and trend visualization.

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub mod connection;
pub mod migrations;
pub mod repository;
pub mod schema;

/// Custom error type for database operations.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    /// Database connection failed
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    /// Migration failed
    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    /// Schema error
    #[error("Schema error: {0}")]
    SchemaError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// SQL error
    #[error("SQL error: {0}")]
    SqlError(#[from] rusqlite::Error),
}

/// Result type for database operations.
pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Manages database connection and operations.
pub struct DatabaseManager {
    db_path: PathBuf,
    connection: Mutex<Connection>,
}

impl DatabaseManager {
    /// Creates a new `DatabaseManager` with the default database path.
    ///
    /// The default path is `~/.local/share/cosmic-applet-opencode-usage/usage.db`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database directory cannot be created
    /// - The database connection cannot be established
    /// - Schema migrations fail
    pub fn new() -> Result<Self> {
        let db_path = Self::default_path()?;
        Self::new_with_path(&db_path)
    }

    /// Creates a new `DatabaseManager` with a custom database path.
    ///
    /// This is primarily useful for testing with temporary databases.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database directory cannot be created
    /// - The database connection cannot be established
    /// - Schema migrations fail
    pub fn new_with_path(path: &Path) -> Result<Self> {
        // Ensure directory exists
        connection::ensure_directory(path)?;

        // Create and configure connection
        let conn = connection::create_connection(path)?;
        connection::configure_connection(&conn)?;

        // Apply migrations
        migrations::apply_migrations(&conn)?;

        Ok(Self {
            db_path: path.to_path_buf(),
            connection: Mutex::new(conn),
        })
    }

    /// Gets a reference to the database connection.
    ///
    /// This returns a `MutexGuard` which will block if another thread is
    /// currently using the connection.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned (i.e., a thread panicked while holding the lock).
    #[must_use = "The connection guard must be used, otherwise the lock is immediately released"]
    pub fn get_connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.connection.lock().expect("Mutex poisoned")
    }

    /// Returns the path to the database file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Returns the default database path.
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    fn default_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").map_err(|e| {
            DatabaseError::ConnectionFailed(format!("Could not determine HOME directory: {e}"))
        })?;

        Ok(PathBuf::from(home)
            .join(".local/share/cosmic-applet-opencode-usage")
            .join("usage.db"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_error_types_exist() {
        // Should compile - just testing error types exist
        let _: DatabaseError;
    }

    #[test]
    fn test_connection_error_display() {
        let err = DatabaseError::ConnectionFailed("test error".to_string());
        let display = err.to_string();
        assert!(display.contains("test error"));
        assert!(display.contains("connection failed"));
    }

    #[test]
    fn test_migration_error_display() {
        let err = DatabaseError::MigrationFailed("migration issue".to_string());
        let display = err.to_string();
        assert!(display.contains("migration issue"));
    }

    #[test]
    fn test_schema_error_display() {
        let err = DatabaseError::SchemaError("schema problem".to_string());
        let display = err.to_string();
        assert!(display.contains("schema problem"));
    }

    #[test]
    fn test_database_manager_new_with_path() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let manager = DatabaseManager::new_with_path(&db_path);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_database_manager_creates_schema() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let manager = DatabaseManager::new_with_path(&db_path).unwrap();
        let conn = manager.get_connection();

        // Verify tables exist
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name != 'sqlite_sequence'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(count >= 2, "Expected at least 2 tables, found {count}");
    }

    #[test]
    fn test_database_manager_get_connection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let manager = DatabaseManager::new_with_path(&db_path).unwrap();
        let _conn = manager.get_connection(); // Should not panic
    }

    #[test]
    fn test_database_manager_path() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let manager = DatabaseManager::new_with_path(&db_path).unwrap();
        assert_eq!(manager.path(), db_path);
    }
}
