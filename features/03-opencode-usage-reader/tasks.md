# Implementation Tasks: OpenCode Usage Reader

## Task 1: Setup Module Structure
**Status**: Pending  
**Type**: Structural

### Description
Create the module file structure for the OpenCode usage reader.

### Steps
1. Create `src/core/opencode/` directory
2. Create `mod.rs`, `scanner.rs`, `parser.rs`, `aggregator.rs`
3. Define public exports in `mod.rs`
4. Add module declaration to `src/core/mod.rs`

### Acceptance Criteria
- [ ] Directory structure created
- [ ] Module files exist
- [ ] Module compiles without errors

---

## Task 2: Define Data Structures (Parser)
**Status**: Pending  
**Type**: Behavioral (with tests)

### Description
Define the data structures for parsing OpenCode usage JSON files.

### Test Cases
1. Test JSON deserialization with complete token data
2. Test JSON deserialization with missing optional fields
3. Test JSON deserialization with cache data
4. Test serialization round-trip

### Implementation
Define in `parser.rs`:
- `UsagePart` struct
- `TokenUsage` struct
- `CacheUsage` struct

### Acceptance Criteria
- [ ] All structs defined with proper serde attributes
- [ ] Tests for deserialization pass
- [ ] Tests for missing fields pass
- [ ] Round-trip serialization works

---

## Task 3: Implement JSON Parser
**Status**: Pending  
**Type**: Behavioral (TDD)

### Test Cases
1. `test_parse_valid_usage_part` - Parse complete JSON successfully
2. `test_parse_part_without_tokens` - Return None for parts without tokens
3. `test_parse_malformed_json` - Return error for invalid JSON
4. `test_parse_empty_file` - Handle empty file gracefully
5. `test_parse_part_with_zero_tokens` - Handle zero token counts

### Implementation
Implement `UsageParser`:
- `parse_json(content: &str) -> Result<Option<UsagePart>, ParserError>`
- `parse_file(path: &Path) -> Result<Option<UsagePart>, ParserError>`

### Acceptance Criteria
- [ ] All test cases pass
- [ ] Parser handles errors gracefully
- [ ] Parser returns None for files without token data
- [ ] Error messages are descriptive

---

## Task 4: Implement Usage Aggregator
**Status**: Pending  
**Type**: Behavioral (TDD)

### Test Cases
1. `test_aggregate_single_part` - Aggregate one usage part correctly
2. `test_aggregate_multiple_parts` - Aggregate multiple parts
3. `test_aggregate_empty` - Handle empty aggregator
4. `test_aggregate_with_cache_tokens` - Correctly count cache tokens
5. `test_interaction_counting` - Count interactions correctly
6. `test_cost_accumulation` - Sum costs accurately

### Implementation
Implement `UsageAggregator`:
- `new() -> Self`
- `add_part(&mut self, part: &UsagePart)`
- `finalize(self) -> UsageMetrics`

Define `UsageMetrics` struct.

### Acceptance Criteria
- [ ] All test cases pass
- [ ] Metrics are calculated correctly
- [ ] Interaction counting works
- [ ] Timestamp is set on finalize

---

## Task 5: Implement Storage Scanner
**Status**: Pending  
**Type**: Behavioral (TDD)

### Test Cases
1. `test_scanner_with_empty_directory` - Handle empty directory
2. `test_scanner_finds_json_files` - Find all JSON files
3. `test_scanner_filters_non_json` - Ignore non-JSON files
4. `test_scanner_nested_directories` - Traverse nested structure
5. `test_scanner_nonexistent_directory` - Return error for missing directory

### Implementation
Implement `StorageScanner`:
- `new() -> Result<Self, ScannerError>`
- `scan(&self) -> Result<Vec<PathBuf>, ScannerError>`

Use `walkdir` crate for traversal.

### Acceptance Criteria
- [ ] All test cases pass
- [ ] Scanner finds JSON files in nested directories
- [ ] Non-JSON files are filtered out
- [ ] Errors are handled appropriately

---

## Task 6: Implement OpenCodeUsageReader (Main Orchestrator)
**Status**: Pending  
**Type**: Behavioral (TDD)

### Test Cases
1. `test_reader_with_sample_data` - Read and aggregate sample data
2. `test_reader_with_no_data` - Handle directory with no usage files
3. `test_reader_caching` - Verify cache is used on second call
4. `test_reader_cache_expiry` - Verify cache expires after 5 minutes
5. `test_reader_skips_invalid_files` - Continue despite invalid JSON files

### Implementation
Implement `OpenCodeUsageReader`:
- `new() -> Result<Self, ReaderError>`
- `get_usage(&mut self) -> Result<UsageMetrics, ReaderError>`
- `scan_and_aggregate(&self) -> Result<UsageMetrics, ReaderError>`
- `should_refresh_cache(&self) -> bool`

### Acceptance Criteria
- [ ] All test cases pass
- [ ] Caching mechanism works correctly
- [ ] Cache expiry is respected
- [ ] Reader handles errors gracefully
- [ ] Invalid files are skipped without failing

---

## Task 7: Integration Testing with Real Data
**Status**: Pending  
**Type**: Integration Test

### Test Cases
1. Create temporary directory mimicking OpenCode storage structure
2. Populate with realistic sample JSON files (10-20 files)
3. Run full scan and aggregation
4. Verify metrics match expected values

### Implementation
Create integration test in `tests/opencode_integration.rs`:
- Setup temp directory with sample data
- Run `OpenCodeUsageReader::get_usage()`
- Assert metrics are correct

### Acceptance Criteria
- [ ] Integration test passes
- [ ] Metrics are accurately calculated
- [ ] Test cleanup is handled properly

---

## Task 8: Error Handling Enhancement
**Status**: Pending  
**Type**: Behavioral

### Test Cases
1. `test_permission_denied` - Handle permission errors
2. `test_concurrent_access` - Handle concurrent file access
3. `test_corrupted_file` - Skip corrupted files gracefully

### Implementation
- Enhance error types
- Add better error context
- Log warnings for skipped files

### Acceptance Criteria
- [ ] All test cases pass
- [ ] Error messages are user-friendly
- [ ] Errors don't crash the reader

---

## Task 9: Remove GitHub API Code
**Status**: Pending  
**Type**: Structural

### Description
Remove `github.rs` and related GitHub API code from the codebase.

### Steps
1. Delete `src/core/github.rs`
2. Remove GitHub-related types from `models.rs`
3. Update `src/core/mod.rs` to remove GitHub module export
4. Remove `reqwest` dependency usage
5. Update any references in UI code

### Acceptance Criteria
- [ ] `github.rs` deleted
- [ ] GitHub models removed
- [ ] Code compiles without errors
- [ ] No unused dependencies

---

## Task 10: Update Configuration for OpenCode
**Status**: Pending  
**Type**: Structural

### Description
Update configuration to support OpenCode-specific settings.

### Steps
1. Remove GitHub token configuration
2. Add OpenCode storage path configuration (optional, with default)
3. Update `config.rs` to reflect changes
4. Remove keyring dependency if no longer needed

### Acceptance Criteria
- [ ] GitHub config removed
- [ ] OpenCode config added
- [ ] Tests updated
- [ ] Configuration loads correctly

---

## Task 11: Update UI to Display OpenCode Metrics
**Status**: Pending  
**Type**: Behavioral

### Description
Update the UI to display OpenCode usage metrics instead of GitHub Copilot metrics.

### Steps
1. Update `state.rs` to use `OpenCodeUsageReader`
2. Modify UI formatters for OpenCode metrics
3. Update panel display with new metrics
4. Remove GitHub-specific UI elements

### Acceptance Criteria
- [ ] UI displays OpenCode metrics correctly
- [ ] Formatting is user-friendly
- [ ] No GitHub references remain in UI
- [ ] Applet compiles and runs

---

## Task 12: Update Localization Strings
**Status**: Pending  
**Type**: Structural

### Description
Update localization files to reflect OpenCode usage tracking.

### Steps
1. Update `i18n/en/cosmic_applet_template.ftl`
2. Update `i18n/nl/cosmic_applet_template.ftl`
3. Change app name references
4. Update metric descriptions

### Acceptance Criteria
- [ ] English strings updated
- [ ] Dutch strings updated (or removed if not maintained)
- [ ] Strings reference OpenCode
- [ ] No GitHub references remain

---

## Development Order

Follow this sequence for optimal TDD workflow:

1. Task 1: Setup Module Structure
2. Task 2: Define Data Structures
3. Task 3: Implement JSON Parser (TDD)
4. Task 4: Implement Usage Aggregator (TDD)
5. Task 5: Implement Storage Scanner (TDD)
6. Task 6: Implement OpenCodeUsageReader (TDD)
7. Task 7: Integration Testing
8. Task 8: Error Handling Enhancement
9. Task 9: Remove GitHub API Code
10. Task 10: Update Configuration
11. Task 11: Update UI
12. Task 12: Update Localization

## Testing Strategy

- Write tests first for each component (Tasks 2-8)
- Use `cargo test` to run all tests
- Use `cargo test --test integration_tests` for integration tests
- Aim for >80% code coverage on core logic

## Notes

- Keep commits small and focused (one task = one commit)
- Run tests after each task completion
- Update documentation as you go
- Follow Rust best practices and idioms
