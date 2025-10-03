// SPDX-License-Identifier: GPL-3.0-only

//! Integration tests for database functionality.

use cosmic_applet_opencode_usage::core::database::DatabaseManager;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

#[test]
fn test_full_database_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create database
    let manager = DatabaseManager::new_with_path(&db_path).unwrap();

    // Verify file exists
    assert!(db_path.exists());

    // Verify schema
    let conn = manager.get_connection();
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(tables.contains(&"usage_snapshots".to_string()));
    assert!(tables.contains(&"schema_version".to_string()));
}

#[test]
fn test_database_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create database and insert data
    {
        let manager = DatabaseManager::new_with_path(&db_path).unwrap();
        let conn = manager.get_connection();

        conn.execute(
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
        )
        .unwrap();
    } // Drop manager and connection

    // Reopen database and verify data persisted
    {
        let manager = DatabaseManager::new_with_path(&db_path).unwrap();
        let conn = manager.get_connection();

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM usage_snapshots", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 1);

        let input_tokens: i64 = conn
            .query_row(
                "SELECT input_tokens FROM usage_snapshots WHERE date = '2024-01-01'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(input_tokens, 1000);
    }
}

#[test]
fn test_concurrent_read_access() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create database and insert test data
    let manager = Arc::new(DatabaseManager::new_with_path(&db_path).unwrap());
    {
        let conn = manager.get_connection();
        conn.execute(
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
        )
        .unwrap();
    }

    // Spawn multiple threads to read concurrently
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let manager = Arc::clone(&manager);
            thread::spawn(move || {
                let conn = manager.get_connection();
                // Perform read operation
                let version: i32 = conn
                    .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                        row.get(0)
                    })
                    .unwrap_or(0);
                assert_eq!(version, 2, "Thread {i} got unexpected version");

                // Also read from usage_snapshots
                let count: i32 = conn
                    .query_row("SELECT COUNT(*) FROM usage_snapshots", [], |row| row.get(0))
                    .unwrap();
                assert_eq!(count, 1, "Thread {i} got unexpected count");
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_write_access() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let manager = Arc::new(DatabaseManager::new_with_path(&db_path).unwrap());

    // Spawn multiple threads to write concurrently
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let manager = Arc::clone(&manager);
            thread::spawn(move || {
                let conn = manager.get_connection();
                conn.execute(
                    "INSERT INTO usage_snapshots 
                     (date, input_tokens, output_tokens, reasoning_tokens, 
                      cache_write_tokens, cache_read_tokens, total_cost, 
                      interaction_count, created_at) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    rusqlite::params![
                        format!("2024-01-{:02}", i + 1),
                        1000 * i64::from(i),
                        2000 * i64::from(i),
                        500 * i64::from(i),
                        100 * i64::from(i),
                        200 * i64::from(i),
                        0.15 * f64::from(i),
                        10 * i,
                        format!("2024-01-{:02}T12:00:00Z", i + 1)
                    ],
                )
                .unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all writes succeeded
    let conn = manager.get_connection();
    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM usage_snapshots", [], |row| row.get(0))
        .unwrap();

    assert_eq!(count, 5);
}

#[test]
fn test_wal_mode_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    let conn = manager.get_connection();

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();

    assert_eq!(journal_mode.to_lowercase(), "wal");
}

#[test]
fn test_foreign_keys_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    let conn = manager.get_connection();

    let foreign_keys: i32 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();

    assert_eq!(foreign_keys, 1);
}

#[test]
fn test_date_index_exists() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    let conn = manager.get_connection();

    let indexes: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(indexes.contains(&"idx_usage_snapshots_date".to_string()));
}

#[test]
fn test_schema_version_recorded() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    let conn = manager.get_connection();

    let version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(version, 2);

    // Verify timestamp is valid
    let applied_at: String = conn
        .query_row(
            "SELECT applied_at FROM schema_version WHERE version = 2",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert!(chrono::DateTime::parse_from_rfc3339(&applied_at).is_ok());
}

#[test]
fn test_all_columns_in_usage_snapshots() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    let conn = manager.get_connection();

    // Get column info
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(usage_snapshots)")
        .unwrap()
        .query_map([], |row| row.get(1))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    let expected_columns = vec![
        "id",
        "date",
        "input_tokens",
        "output_tokens",
        "reasoning_tokens",
        "cache_write_tokens",
        "cache_read_tokens",
        "total_cost",
        "interaction_count",
        "created_at",
    ];

    for expected in expected_columns {
        assert!(
            columns.contains(&expected.to_string()),
            "Missing column: {expected}"
        );
    }
}

#[test]
fn test_database_manager_path_method() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    assert_eq!(manager.path(), db_path);
}

#[test]
fn test_reopening_database_preserves_schema() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Create and close
    {
        DatabaseManager::new_with_path(&db_path).unwrap();
    }

    // Reopen
    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    let conn = manager.get_connection();

    // Verify schema is still intact
    let version: i32 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(version, 2);
}
