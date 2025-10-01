// SPDX-License-Identifier: GPL-3.0-only

//! Integration tests for the viewer application.

use cosmic_applet_opencode_usage::core::database::{repository::UsageRepository, DatabaseManager};
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn test_viewer_database_connection() {
    // Create temporary database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("viewer_test.db");

    // Initialize DatabaseManager
    let database_manager = DatabaseManager::new_with_path(&db_path);
    assert!(database_manager.is_ok(), "Database initialization failed");

    let manager = database_manager.unwrap();

    // Verify connection works by executing a simple query
    let conn = manager.get_connection();
    let result: Result<i32, _> = conn.query_row("SELECT 1", [], |row| row.get(0));
    assert!(result.is_ok(), "Database connection test query failed");
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_viewer_repository_access() {
    // Create temporary database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("viewer_repo_test.db");

    // Initialize database and repository
    let database_manager = Arc::new(DatabaseManager::new_with_path(&db_path).unwrap());
    let repository = UsageRepository::new(Arc::clone(&database_manager));

    // Verify repository can access database
    // Try to get latest snapshot (should return None for new database)
    let latest = repository.get_latest();
    assert!(latest.is_ok(), "Repository access failed");
    assert!(
        latest.unwrap().is_none(),
        "New database should have no snapshots"
    );
}

#[test]
#[ignore = "Documentation test - verified by cargo build"]
fn test_viewer_binary_compiles() {
    // This test verifies that the binary target exists and can be built
    // The actual compilation is verified by the build system
    // This is a placeholder test to document the requirement
}

#[test]
fn test_database_shared_with_applet() {
    // Verify that viewer and applet can share the same database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("shared_test.db");

    // Create database with "applet" (using DatabaseManager)
    let manager1 = Arc::new(DatabaseManager::new_with_path(&db_path).unwrap());
    let repo1 = UsageRepository::new(Arc::clone(&manager1));

    // Verify we can read data
    let latest = repo1.get_latest();
    assert!(latest.is_ok());

    // Drop the first connection
    drop(repo1);
    drop(manager1);

    // Create new connection with "viewer" (simulating separate process)
    let manager2 = Arc::new(DatabaseManager::new_with_path(&db_path).unwrap());
    let repo2 = UsageRepository::new(Arc::clone(&manager2));

    // Verify we can read data
    let latest = repo2.get_latest();
    assert!(
        latest.is_ok(),
        "Viewer should be able to access shared database"
    );
}

#[test]
#[ignore = "Documentation test - verified by full test suite"]
fn test_no_applet_regression() {
    // Verify that adding the viewer doesn't break existing applet functionality
    // This is a meta-test that documents the requirement
    // The actual verification is done by running the full test suite
}
