# Data Collection Implementation Tasks

## TDD Methodology

Follow Red → Green → Refactor cycle:
1. **RED**: Write failing test for next functionality
2. **GREEN**: Write minimal code to pass test
3. **REFACTOR**: Improve code while keeping tests green

Run `cargo test` after each step.

## Task Breakdown

### Phase 1: Repository Foundation

#### Task 1.1: Create UsageSnapshot struct
- **Test**: `test_usage_snapshot_creation`
  - Create UsageSnapshot with test data
  - Assert all fields are correctly set
- **Implementation**:
  - Create `src/core/database/repository.rs`
  - Define `UsageSnapshot` struct with all fields
  - Derive Debug, Clone
  - Add SPDX header
- **Refactor**: Add documentation

#### Task 1.2: Create UsageRepository struct
- **Test**: `test_repository_creation`
  - Create repository with test database
  - Assert repository is created successfully
- **Implementation**:
  - Define `UsageRepository` struct with db field
  - Implement `new()` constructor
  - Add to mod.rs exports
- **Refactor**: Add documentation

#### Task 1.3: Implement save_snapshot
- **Test**: `test_save_snapshot`
  - Save snapshot with test data
  - Verify no error returned
- **Test**: `test_save_snapshot_duplicate_date`
  - Save snapshot for date
  - Save again with different metrics
  - Verify second save succeeds (REPLACE)
- **Implementation**:
  - Implement `save_snapshot()` method
  - Use INSERT OR REPLACE SQL
  - Handle database errors
- **Refactor**: Extract SQL query constants

#### Task 1.4: Implement get_snapshot
- **Test**: `test_get_snapshot_exists`
  - Save snapshot
  - Retrieve by date
  - Assert Some(snapshot) with correct data
- **Test**: `test_get_snapshot_missing`
  - Query non-existent date
  - Assert None returned
- **Implementation**:
  - Implement `get_snapshot()` method
  - Query by date
  - Convert row to UsageSnapshot
- **Refactor**: Extract row conversion logic

#### Task 1.5: Implement get_range
- **Test**: `test_get_range_multiple`
  - Save 5 snapshots across dates
  - Get range for middle 3
  - Assert 3 snapshots returned in order
- **Test**: `test_get_range_empty`
  - Query range with no data
  - Assert empty Vec returned
- **Implementation**:
  - Implement `get_range()` method
  - Query with date range
  - Sort by date ascending
- **Refactor**: Optimize query with prepared statement

#### Task 1.6: Implement get_latest
- **Test**: `test_get_latest_exists`
  - Save 3 snapshots with different dates
  - Get latest
  - Assert most recent date returned
- **Test**: `test_get_latest_empty`
  - Query empty database
  - Assert None returned
- **Implementation**:
  - Implement `get_latest()` method
  - Query with ORDER BY date DESC LIMIT 1
- **Refactor**: Add caching if beneficial

#### Task 1.7: Implement delete_old
- **Test**: `test_delete_old_removes_old`
  - Save snapshots for 10 days ago, 5 days ago, today
  - Delete older than 7 days
  - Assert 1 deleted, 2 remain
- **Test**: `test_delete_old_empty_database`
  - Delete from empty database
  - Assert 0 deleted
- **Implementation**:
  - Implement `delete_old()` method
  - Calculate cutoff date
  - DELETE WHERE date < cutoff
  - Return count deleted
- **Refactor**: Add logging for deleted count

### Phase 2: Data Collector

#### Task 2.1: Create DataCollector struct
- **Test**: `test_collector_creation`
  - Create collector with test database
  - Assert collector created successfully
- **Implementation**:
  - Create `src/core/collector/mod.rs`
  - Define `DataCollector` struct
  - Add repository and last_collection fields
  - Implement `new()` constructor
- **Refactor**: Add documentation

#### Task 2.2: Implement should_collect
- **Test**: `test_should_collect_first_time`
  - Create collector
  - Assert should_collect() returns true
- **Test**: `test_should_collect_same_day`
  - Simulate collection today
  - Assert should_collect() returns false
- **Test**: `test_should_collect_next_day`
  - Simulate collection yesterday
  - Assert should_collect() returns true
- **Implementation**:
  - Implement `should_collect()` method
  - Get current date (UTC)
  - Compare with last_collection date
  - Return true if different or None
- **Refactor**: Extract date comparison logic

#### Task 2.3: Implement get_last_collection_date
- **Test**: `test_get_last_collection_none`
  - Create new collector
  - Assert None returned
- **Test**: `test_get_last_collection_some`
  - Set last collection date
  - Assert correct date returned
- **Implementation**:
  - Implement `get_last_collection_date()` method
  - Lock and read last_collection
  - Return cloned Option
- **Refactor**: Handle lock errors gracefully

#### Task 2.4: Implement collect_and_save
- **Test**: `test_collect_and_save_first_time`
  - Create collector
  - Call collect_and_save with metrics
  - Assert returns Ok(true)
  - Verify snapshot saved in database
- **Test**: `test_collect_and_save_same_day_twice`
  - Call collect_and_save twice with same metrics
  - First returns Ok(true)
  - Second returns Ok(false)
  - Verify only one snapshot in database
- **Test**: `test_collect_and_save_different_days`
  - Call collect_and_save with metrics
  - Simulate date change
  - Call again
  - Both return Ok(true)
  - Verify two snapshots in database
- **Implementation**:
  - Implement `collect_and_save()` method
  - Get current date
  - Lock last_collection
  - Check if should collect
  - If yes: save snapshot, update last_collection, return true
  - If no: return false
- **Refactor**: Extract date handling logic

#### Task 2.5: Error handling
- **Test**: `test_collect_database_error`
  - Mock database error
  - Call collect_and_save
  - Assert error propagated
- **Test**: `test_concurrent_collect`
  - Call collect_and_save from multiple threads
  - Assert thread-safe behavior
- **Implementation**:
  - Define CollectorError enum
  - Add proper error conversion
  - Handle lock poisoning
- **Refactor**: Improve error messages

### Phase 3: App Integration

#### Task 3.1: Add DataCollector to Window
- **Test**: `test_window_with_collector` (manual/integration)
  - Initialize window
  - Verify collector initialized if database available
- **Implementation**:
  - Add `data_collector: Option<DataCollector>` to Window struct
  - Initialize in `init()` or appropriate method
  - Handle database initialization failure gracefully
- **Refactor**: Clean up initialization code

#### Task 3.2: Integrate with UsageFetched
- **Test**: `test_save_on_usage_fetched` (integration)
  - Trigger UsageFetched message
  - Verify collect_and_save called
  - Check database for snapshot
- **Implementation**:
  - In `update()` method for UsageFetched message
  - Call `data_collector.collect_and_save(&metrics)`
  - Log errors but don't crash
- **Refactor**: Extract snapshot logic to helper method

#### Task 3.3: Add SaveSnapshot message (optional)
- **Test**: `test_save_snapshot_message` (integration)
  - Send SaveSnapshot message
  - Verify collection attempted
- **Implementation**:
  - Add `SaveSnapshot` to Message enum
  - Handle in update() method
  - Add timer subscription (optional)
- **Refactor**: Make timer configurable

### Phase 4: Integration & Cleanup

#### Task 4.1: Integration tests
- **Test**: `test_full_collection_workflow`
  - Initialize database
  - Create collector
  - Simulate multiple days of usage
  - Verify all snapshots saved correctly
- **Test**: `test_applet_restart_scenario`
  - Save snapshot
  - Drop collector
  - Create new collector
  - Attempt save same day
  - Verify no duplicate
- **Implementation**:
  - Add tests to `tests/database_integration.rs`
  - Test realistic scenarios
- **Refactor**: Add test helpers

#### Task 4.2: Module exports
- **Test**: Verify all public APIs accessible
- **Implementation**:
  - Export repository in `src/core/database/mod.rs`
  - Export collector in `src/core/mod.rs`
  - Add re-exports as needed
- **Refactor**: Organize exports logically

#### Task 4.3: Documentation
- **Implementation**:
  - Add module-level documentation
  - Document all public functions
  - Add usage examples
- **Refactor**: Improve clarity

#### Task 4.4: Code quality
- **Implementation**:
  - Run `cargo fmt`
  - Run `cargo clippy -- -W clippy::pedantic`
  - Fix all warnings
  - Ensure SPDX headers on all files
- **Refactor**: Address any clippy suggestions

## Acceptance Criteria

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] `cargo clippy` reports no warnings
- [ ] `cargo fmt` requires no changes
- [ ] Code coverage > 80% for new code
- [ ] All public APIs documented
- [ ] SPDX headers on all new files
- [ ] Collector initializes in app
- [ ] Snapshots saved automatically on usage fetch
- [ ] No duplicate snapshots for same day
- [ ] Errors logged but don't crash applet
- [ ] Database operations complete quickly (< 100ms)

## Testing Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_save_snapshot

# Run with output
cargo test -- --nocapture

# Run clippy
cargo clippy --all-features -- -W clippy::pedantic

# Format code
cargo +nightly fmt --all

# Build release
just build-release
```

## Implementation Notes

- **Test files**: Place in `tests/` for integration, within module for unit tests
- **Mock database**: Use in-memory SQLite (`:memory:`) for tests
- **Date handling**: Use `chrono::Utc::now().date_naive()` for current date
- **Thread safety**: Use `Arc<Mutex<>>` for shared state
- **Error context**: Use `.map_err()` to add context to errors
- **Logging**: Use `eprintln!` for errors (or proper logging crate if available)

## Estimated Time

- Phase 1: 2-3 hours (repository implementation)
- Phase 2: 2-3 hours (collector implementation)
- Phase 3: 1-2 hours (app integration)
- Phase 4: 1 hour (cleanup and documentation)

**Total**: 6-9 hours of focused development
