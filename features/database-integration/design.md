# Database Integration Design

## Architecture Overview

The database integration will be implemented as a new module `src/core/database/` with clear separation of concerns:

```
src/core/database/
├── mod.rs          # Public API and DatabaseManager
├── schema.rs       # Table definitions and SQL statements
├── connection.rs   # Connection management and pooling
└── migrations.rs   # Migration logic and versioning
```

## Technical Design

### 1. Database Manager (`mod.rs`)

**Purpose**: Provide high-level API for database operations.

**Key Components**:
```rust
pub struct DatabaseManager {
    db_path: PathBuf,
    connection: Mutex<Connection>,
}

impl DatabaseManager {
    pub fn new() -> Result<Self, DatabaseError>;
    pub fn get_connection(&self) -> MutexGuard<Connection>;
    pub fn ensure_schema() -> Result<(), DatabaseError>;
}
```

**Design Decisions**:
- Use `Mutex<Connection>` for thread-safe access
- Database path: `~/.local/share/cosmic-applet-opencode-usage/usage.db`
- Initialize database on `new()` call
- Lazy migration application

### 2. Schema Definitions (`schema.rs`)

**Purpose**: Centralize all SQL schema definitions.

**Tables**:

1. **usage_snapshots**:
   - Stores daily usage metrics snapshots
   - Primary key: auto-incrementing id
   - Indexed on: date (for efficient queries)

2. **schema_version**:
   - Tracks applied migrations
   - Fields: version (INTEGER), applied_at (TEXT)

**SQL Constants**:
```rust
pub const CREATE_USAGE_SNAPSHOTS_TABLE: &str = "...";
pub const CREATE_SCHEMA_VERSION_TABLE: &str = "...";
pub const CREATE_DATE_INDEX: &str = "...";
```

### 3. Connection Management (`connection.rs`)

**Purpose**: Handle database connection lifecycle.

**Features**:
- Set WAL mode for better concurrency
- Configure pragmas (foreign_keys, synchronous)
- Ensure directory structure exists
- Handle connection errors gracefully

**Key Functions**:
```rust
pub fn create_connection(path: &Path) -> Result<Connection, DatabaseError>;
pub fn ensure_directory(path: &Path) -> Result<(), DatabaseError>;
pub fn configure_connection(conn: &Connection) -> Result<(), DatabaseError>;
```

### 4. Migrations (`migrations.rs`)

**Purpose**: Handle schema versioning and updates.

**Migration Strategy**:
- Each migration has a version number
- Migrations are idempotent (can be re-run safely)
- Use transactions for atomic application
- Track applied migrations in `schema_version` table

**Key Functions**:
```rust
pub struct Migration {
    version: i32,
    description: String,
    sql: String,
}

pub fn get_migrations() -> Vec<Migration>;
pub fn apply_migrations(conn: &Connection) -> Result<(), DatabaseError>;
pub fn get_current_version(conn: &Connection) -> Result<i32, DatabaseError>;
```

## Error Handling

**Custom Error Type**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Schema error: {0}")]
    SchemaError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("SQL error: {0}")]
    SqlError(#[from] rusqlite::Error),
}
```

## Data Flow

1. **Initialization**:
   ```
   App Start → DatabaseManager::new()
           ↓
   Create directory if needed
           ↓
   Create/Open connection
           ↓
   Configure pragmas (WAL, etc.)
           ↓
   Apply migrations
           ↓
   Ready for operations
   ```

2. **Query Execution**:
   ```
   Request → get_connection()
         ↓
   Acquire Mutex lock
         ↓
   Execute SQL
         ↓
   Release lock
         ↓
   Return result
   ```

## Integration Points

### With Existing Code

1. **config.rs**: Store database path in config
2. **app.rs**: Initialize DatabaseManager in app state
3. **opencode/**: Future integration for persisting parsed metrics

### Future Extensions

1. **Query API**: Add methods for reading snapshots
2. **Aggregation**: Support for time-based aggregations
3. **Cleanup**: Automatic old data pruning
4. **Export**: Export data to JSON/CSV

## Testing Strategy

### Unit Tests
- Schema creation
- Migration application
- Error handling
- Connection configuration

### Integration Tests
- Full database lifecycle
- Concurrent access patterns
- Migration rollback scenarios
- Data integrity verification

### Test Database Location
- Use temp directories for tests
- Clean up after each test
- Avoid test interference

## Performance Considerations

1. **WAL Mode**: Enables concurrent reads during writes
2. **Indexes**: Date column indexed for efficient queries
3. **Connection Reuse**: Single connection with mutex (sufficient for applet)
4. **Batch Operations**: Future support for bulk inserts

## Security Considerations

1. **File Permissions**: Database file user-only (600)
2. **SQL Injection**: Use parameterized queries only
3. **Path Traversal**: Validate database path
4. **Data Privacy**: Local storage only, no network exposure

## Dependencies

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
thiserror = "2.0"
```

**Rationale**:
- `bundled` feature: Include SQLite library (no external dependency)
- `rusqlite` 0.32: Latest stable with good API
- `thiserror`: Idiomatic error handling

## Rollout Plan

1. Implement core database module (this feature)
2. Add CRUD operations for usage snapshots
3. Integrate with parser to persist metrics
4. Add UI for historical data visualization
5. Implement data export functionality
