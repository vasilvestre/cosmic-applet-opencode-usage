# Database Integration Implementation Tasks

## TDD Task Breakdown

### Task 1: Setup Dependencies and Module Structure
**Goal**: Add required dependencies and create module skeleton.

**Steps**:
1. Add `rusqlite` to Cargo.toml with `bundled` feature
2. Create `src/core/database/` directory
3. Create module files: `mod.rs`, `schema.rs`, `connection.rs`, `migrations.rs`
4. Add `pub mod database;` to `src/core/mod.rs`

**Acceptance Criteria**:
- [ ] Project compiles with new dependency
- [ ] Module structure is accessible

---

### Task 2: Define Error Types (TDD)
**Goal**: Create custom error type for database operations.

**Red Phase**:
```rust
// tests/database_errors.rs
#[test]
fn test_error_types_exist() {
    use cosmic_applet_opencode_usage::core::database::DatabaseError;
    // Should compile
}

#[test]
fn test_connection_error_display() {
    let err = DatabaseError::ConnectionFailed("test".to_string());
    assert!(err.to_string().contains("test"));
}
```

**Green Phase**:
- Implement `DatabaseError` enum in `mod.rs`
- Use `thiserror` for derive macro
- Include variants: ConnectionFailed, MigrationFailed, SchemaError, IoError, SqlError

**Refactor Phase**:
- Ensure error messages are clear and actionable
- Add `#[from]` conversions where appropriate

**Acceptance Criteria**:
- [ ] All error tests pass
- [ ] Error messages are descriptive

---

### Task 3: Schema Definitions (TDD)
**Goal**: Define SQL schema constants.

**Red Phase**:
```rust
// tests/database_schema.rs
#[test]
fn test_usage_snapshots_table_sql() {
    use cosmic_applet_opencode_usage::core::database::schema::*;
    assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("usage_snapshots"));
    assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("input_tokens"));
}

#[test]
fn test_schema_version_table_sql() {
    assert!(CREATE_SCHEMA_VERSION_TABLE.contains("schema_version"));
}
```

**Green Phase**:
- Create `schema.rs` with SQL constants
- Define `CREATE_USAGE_SNAPSHOTS_TABLE`
- Define `CREATE_SCHEMA_VERSION_TABLE`
- Define `CREATE_DATE_INDEX`

**Refactor Phase**:
- Format SQL for readability
- Add comments explaining schema design

**Acceptance Criteria**:
- [ ] SQL strings are syntactically valid
- [ ] All required columns included
- [ ] Tests pass

---

### Task 4: Connection Management (TDD)
**Goal**: Implement database connection creation and configuration.

**Red Phase**:
```rust
// tests/database_connection.rs
#[test]
fn test_create_connection_in_memory() {
    let conn = create_connection(Path::new(":memory:"));
    assert!(conn.is_ok());
}

#[test]
fn test_configure_connection_enables_wal() {
    let conn = Connection::open_in_memory().unwrap();
    configure_connection(&conn).unwrap();
    
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_mode.to_lowercase(), "wal");
}

#[test]
fn test_ensure_directory_creates_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("subdir/db.sqlite");
    ensure_directory(&db_path).unwrap();
    assert!(db_path.parent().unwrap().exists());
}
```

**Green Phase**:
- Implement `create_connection()` in `connection.rs`
- Implement `configure_connection()` (WAL mode, foreign keys, etc.)
- Implement `ensure_directory()` using `std::fs::create_dir_all`

**Refactor Phase**:
- Extract pragma configuration to constants
- Improve error messages
- Add documentation

**Acceptance Criteria**:
- [ ] Connection creation succeeds
- [ ] WAL mode is enabled
- [ ] Directory creation works
- [ ] All tests pass

---

### Task 5: Migration System (TDD)
**Goal**: Implement schema migration logic.

**Red Phase**:
```rust
// tests/database_migrations.rs
#[test]
fn test_get_migrations_returns_list() {
    let migrations = get_migrations();
    assert!(!migrations.is_empty());
    assert_eq!(migrations[0].version, 1);
}

#[test]
fn test_get_current_version_on_new_db() {
    let conn = Connection::open_in_memory().unwrap();
    let version = get_current_version(&conn).unwrap();
    assert_eq!(version, 0);
}

#[test]
fn test_apply_migrations_creates_tables() {
    let conn = Connection::open_in_memory().unwrap();
    apply_migrations(&conn).unwrap();
    
    // Check tables exist
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    
    assert!(tables.contains(&"usage_snapshots".to_string()));
    assert!(tables.contains(&"schema_version".to_string()));
}

#[test]
fn test_apply_migrations_is_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    apply_migrations(&conn).unwrap();
    apply_migrations(&conn).unwrap(); // Should not error
}
```

**Green Phase**:
- Define `Migration` struct in `migrations.rs`
- Implement `get_migrations()` returning initial migration
- Implement `get_current_version()` checking schema_version table
- Implement `apply_migrations()` with transaction support

**Refactor Phase**:
- Ensure idempotency
- Add rollback on migration failure
- Improve error messages with migration context

**Acceptance Criteria**:
- [ ] Migrations can be listed
- [ ] Current version is tracked
- [ ] Migrations apply successfully
- [ ] Idempotent (safe to re-run)
- [ ] All tests pass

---

### Task 6: Database Manager (TDD)
**Goal**: Implement high-level DatabaseManager API.

**Red Phase**:
```rust
// tests/database_manager.rs
#[test]
fn test_database_manager_new() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let manager = DatabaseManager::new_with_path(&db_path);
    assert!(manager.is_ok());
}

#[test]
fn test_database_manager_creates_schema() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    let conn = manager.get_connection();
    
    // Verify tables exist
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0)
        )
        .unwrap();
    
    assert!(count >= 2); // At least usage_snapshots and schema_version
}

#[test]
fn test_database_manager_get_connection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    let _conn = manager.get_connection(); // Should not panic
}
```

**Green Phase**:
- Define `DatabaseManager` struct in `mod.rs`
- Implement `new()` using default path
- Implement `new_with_path()` for testing
- Implement `get_connection()` returning `MutexGuard<Connection>`
- Implement `ensure_schema()` calling migration logic

**Refactor Phase**:
- Add constructor documentation
- Simplify initialization logic
- Ensure proper error propagation

**Acceptance Criteria**:
- [ ] DatabaseManager initializes correctly
- [ ] Schema is created on initialization
- [ ] Connection can be acquired
- [ ] Default path works correctly
- [ ] All tests pass

---

### Task 7: Concurrent Access Testing (TDD)
**Goal**: Verify thread-safe database access.

**Red Phase**:
```rust
// tests/database_concurrent.rs
#[test]
fn test_concurrent_read_access() {
    use std::sync::Arc;
    use std::thread;
    
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let manager = Arc::new(DatabaseManager::new_with_path(&db_path).unwrap());
    
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let manager = Arc::clone(&manager);
            thread::spawn(move || {
                let conn = manager.get_connection();
                // Perform read operation
                let version: i32 = conn
                    .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                        row.get(0)
                    })
                    .unwrap_or(0);
                assert!(version > 0);
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

**Green Phase**:
- Ensure `Mutex<Connection>` is used correctly
- Verify no deadlocks occur
- Test passes with concurrent access

**Refactor Phase**:
- Add documentation about thread safety
- Consider connection pooling for future optimization

**Acceptance Criteria**:
- [ ] Multiple threads can access database
- [ ] No race conditions
- [ ] Tests pass reliably

---

### Task 8: Integration Testing
**Goal**: End-to-end database lifecycle testing.

**Red Phase**:
```rust
// tests/database_integration.rs
#[test]
fn test_full_database_lifecycle() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Create database
    let manager = DatabaseManager::new_with_path(&db_path).unwrap();
    
    // Verify file exists
    assert!(db_path.exists());
    
    // Verify schema
    let conn = manager.get_connection();
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    
    assert!(tables.contains(&"usage_snapshots".to_string()));
}
```

**Green Phase**:
- Ensure all components work together
- Verify database file is created correctly
- Test full initialization flow

**Refactor Phase**:
- Clean up test utilities
- Add helper functions for common test operations

**Acceptance Criteria**:
- [ ] Full lifecycle works end-to-end
- [ ] Database file created at correct location
- [ ] All integration tests pass

---

### Task 9: Code Quality and Documentation
**Goal**: Ensure code meets project standards.

**Steps**:
1. Run `cargo fmt` on all new files
2. Run `cargo clippy` and fix all warnings
3. Add module-level documentation
4. Add function-level documentation with examples
5. Add SPDX headers to all new files

**Acceptance Criteria**:
- [ ] No clippy warnings
- [ ] Code is formatted
- [ ] All public items documented
- [ ] SPDX headers present

---

### Task 10: Final Verification
**Goal**: Comprehensive testing and validation.

**Steps**:
1. Run `cargo test` - all tests pass
2. Run `cargo test --doc` - documentation tests pass
3. Run `cargo build --release` - successful build
4. Verify no unwrap() in production code paths
5. Review error handling coverage

**Acceptance Criteria**:
- [ ] All tests pass (100%)
- [ ] No panics in normal operation
- [ ] Build succeeds without warnings
- [ ] Error handling is comprehensive

---

## Summary

This TDD approach ensures:
- **Test Coverage**: Every feature tested before implementation
- **Incremental Progress**: Small, manageable steps
- **Quality**: Built-in validation at each stage
- **Documentation**: Tests serve as usage examples

Total estimated implementation time: 3-4 hours following TDD discipline.
