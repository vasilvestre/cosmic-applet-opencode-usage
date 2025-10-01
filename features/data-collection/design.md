# Data Collection Design

## Architecture Overview

The data collection system consists of three main components:

```
┌─────────────────┐
│   App (UI)      │
│  src/app.rs     │
└────────┬────────┘
         │ calls collect_and_save()
         ▼
┌─────────────────┐
│  DataCollector  │
│  collector/     │  ← Business logic: when to collect
└────────┬────────┘
         │ uses
         ▼
┌─────────────────┐
│ UsageRepository │
│  repository.rs  │  ← Data access: CRUD operations
└────────┬────────┘
         │ uses
         ▼
┌─────────────────┐
│ DatabaseManager │
│  database/      │  ← Connection & schema
└─────────────────┘
```

## Component Design

### 1. UsageRepository (src/core/database/repository.rs)

**Purpose**: Provide high-level data access operations for usage snapshots.

**Interface**:
```rust
pub struct UsageRepository {
    db: Arc<DatabaseManager>,
}

impl UsageRepository {
    pub fn new(db: Arc<DatabaseManager>) -> Self;
    
    // Save a snapshot for a specific date
    pub fn save_snapshot(&self, date: NaiveDate, metrics: &UsageMetrics) 
        -> Result<(), DatabaseError>;
    
    // Get snapshot for a specific date
    pub fn get_snapshot(&self, date: NaiveDate) 
        -> Result<Option<UsageSnapshot>, DatabaseError>;
    
    // Get snapshots within date range (inclusive)
    pub fn get_range(&self, start: NaiveDate, end: NaiveDate) 
        -> Result<Vec<UsageSnapshot>, DatabaseError>;
    
    // Get the most recent snapshot
    pub fn get_latest(&self) 
        -> Result<Option<UsageSnapshot>, DatabaseError>;
    
    // Delete snapshots older than specified days
    pub fn delete_old(&self, days: u32) 
        -> Result<usize, DatabaseError>;
}

pub struct UsageSnapshot {
    pub date: NaiveDate,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
    pub total_cost: f64,
}
```

**Implementation Details**:
- Uses prepared SQL statements for performance
- Handles INSERT OR REPLACE for idempotency
- Converts between database rows and Rust structs
- Propagates database errors with context

### 2. DataCollector (src/core/collector/mod.rs)

**Purpose**: Manage collection timing and business logic.

**Interface**:
```rust
pub struct DataCollector {
    repository: UsageRepository,
    last_collection: Arc<Mutex<Option<NaiveDate>>>,
}

impl DataCollector {
    pub fn new(db: Arc<DatabaseManager>) -> Self;
    
    // Collect and save if needed
    pub fn collect_and_save(&self, metrics: &UsageMetrics) 
        -> Result<bool, CollectorError>;
    
    // Check if collection should happen
    pub fn should_collect(&self) -> bool;
    
    // Get last collection date
    pub fn get_last_collection_date(&self) -> Option<NaiveDate>;
}
```

**Logic Flow**:
1. When `collect_and_save()` is called:
   - Get current date (UTC)
   - Check last collection date
   - If different day or first collection:
     - Save snapshot via repository
     - Update last collection date
     - Return true (collected)
   - Otherwise return false (skipped)

2. Thread-safe tracking of last collection date using Arc<Mutex<>>

### 3. App Integration (src/app.rs)

**Changes Required**:

```rust
pub struct Window {
    // Existing fields...
    data_collector: Option<DataCollector>,
}

// Add new message
pub enum Message {
    // Existing messages...
    SaveSnapshot,
}

// In init():
let data_collector = DatabaseManager::new()
    .ok()
    .map(|db| DataCollector::new(Arc::new(db)));

// In update() for UsageFetched:
if let Some(collector) = &self.data_collector {
    if let Err(e) = collector.collect_and_save(&metrics) {
        eprintln!("Failed to save snapshot: {}", e);
    }
}

// Optional: Add daily timer
// In subscription():
cosmic::iced::time::every(Duration::from_secs(3600))
    .map(|_| Message::SaveSnapshot)
```

## Data Flow

### Collection Flow
```
User opens applet
    ↓
App fetches metrics from OpenCode
    ↓
UsageFetched message received
    ↓
DataCollector.collect_and_save() called
    ↓
Check: Is today different from last collection?
    ↓ YES                           ↓ NO
Save via repository              Skip (return false)
Update last collection date
Return true
```

### Storage Flow
```
Repository.save_snapshot(date, metrics)
    ↓
Get database connection
    ↓
Execute INSERT OR REPLACE
    ↓
Commit transaction
    ↓
Return success/error
```

## Database Schema

Using existing `usage_snapshots` table:

```sql
CREATE TABLE usage_snapshots (
    id INTEGER PRIMARY KEY,
    date TEXT NOT NULL UNIQUE,
    total_tokens INTEGER NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    cache_creation_tokens INTEGER NOT NULL,
    cache_read_tokens INTEGER NOT NULL,
    total_cost REAL NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_date ON usage_snapshots(date DESC);
```

## Error Handling

### Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum CollectorError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Failed to acquire lock")]
    LockError,
}
```

### Error Strategy
- Database errors: Log and continue (don't crash applet)
- Lock errors: Log and retry on next collection
- Missing data: Return None, not an error
- Invalid dates: Return validation error

## Testing Strategy

### Unit Tests (repository.rs)
1. `test_save_snapshot` - Basic save operation
2. `test_save_duplicate_date` - INSERT OR REPLACE behavior
3. `test_get_snapshot_exists` - Retrieve existing data
4. `test_get_snapshot_missing` - Return None for missing
5. `test_get_range` - Multiple snapshots in range
6. `test_get_range_empty` - No data in range
7. `test_get_latest` - Most recent snapshot
8. `test_get_latest_empty` - No snapshots exist
9. `test_delete_old` - Cleanup old snapshots
10. `test_delete_old_preserves_recent` - Keep recent data

### Integration Tests (collector tests)
1. `test_collect_first_time` - First collection succeeds
2. `test_collect_same_day_twice` - Second call skips
3. `test_collect_next_day` - New day triggers collection
4. `test_should_collect_logic` - Business logic correctness
5. `test_concurrent_collection` - Thread safety
6. `test_error_handling` - Database error propagation

### Integration Tests (database_integration.rs)
1. `test_data_collection_workflow` - End-to-end flow
2. `test_applet_restart_no_duplicate` - Restart handling

## Performance Considerations

- **Lazy initialization**: Create DataCollector only if database succeeds
- **Non-blocking**: Database operations don't block UI
- **Prepared statements**: Reuse SQL statements for efficiency
- **Indexed queries**: Use date index for fast lookups
- **Minimal locking**: Lock only last_collection_date, not database operations

## Security & Privacy

- Data stored locally only (SQLite)
- No network transmission
- Standard file permissions (user-only)
- No sensitive user data (only token counts)

## Future Enhancements

1. Configurable retention period
2. Export data to CSV/JSON
3. Aggregate statistics (weekly/monthly)
4. Automatic cleanup on startup
5. Configurable collection frequency
