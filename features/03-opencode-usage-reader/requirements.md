# Feature: OpenCode Usage Reader

## Overview
Develop a Rust module to read and parse OpenCode usage data from local storage files (`~/.local/share/opencode/storage/part/`), aggregate token usage metrics, and provide structured data for display in the COSMIC applet.

## Requirements

### Storage Access
- The system SHALL read usage data from `~/.local/share/opencode/storage/part/` directory
- The system SHALL locate the user's home directory using standard environment variables
- The system SHALL handle missing or inaccessible storage directories gracefully
- WHEN the storage directory does not exist THEN the system SHALL return an appropriate error

### Data Discovery
- The system SHALL recursively traverse the part storage directory structure
- The system SHALL identify JSON files containing usage data
- The system SHALL filter for files containing `tokens` field data
- The system SHALL handle deeply nested directory structures efficiently

### Data Parsing
- The system SHALL parse individual JSON files into strongly-typed Rust structs
- The system SHALL extract the following fields from each part file:
  - `id`: Part identifier
  - `messageID`: Associated message identifier
  - `sessionID`: Associated session identifier
  - `type`: Event type (e.g., "step-finish")
  - `tokens`: Token usage object containing:
    - `input`: Input token count
    - `output`: Output token count
    - `reasoning`: Reasoning token count
    - `cache`: Cache metrics object containing:
      - `write`: Cache write token count
      - `read`: Cache read token count
  - `cost`: Cost in dollars (numeric)
- WHEN JSON parsing fails THEN the system SHALL log the error and skip the file
- WHEN required fields are missing THEN the system SHALL skip the file

### Data Aggregation
- The system SHALL aggregate token usage across all parsed files
- The system SHALL calculate the following totals:
  - Total input tokens
  - Total output tokens
  - Total reasoning tokens
  - Total cache write tokens
  - Total cache read tokens
  - Total cost
- The system SHALL count the total number of interactions (step-finish events)
- The system SHALL support filtering by date range (optional)
- The system SHALL support filtering by session ID (optional)

### Performance
- The system SHALL process files asynchronously to avoid blocking the UI
- The system SHALL limit memory usage by processing files in batches
- The system SHALL complete initial scan within 5 seconds for typical usage (< 100k files)
- WHILE scanning files the system SHALL provide progress indication

### Caching
- The system SHALL cache aggregated results to avoid repeated file system scans
- The system SHALL invalidate cache when new usage data is detected
- The system SHALL store cache with timestamp of last scan
- The system SHALL refresh cache automatically every 5 minutes when applet is active

### Error Handling
- The system SHALL handle file system permission errors
- The system SHALL handle corrupted or malformed JSON files
- The system SHALL handle concurrent file access issues
- The system SHALL provide meaningful error messages for troubleshooting
- WHEN file system errors occur THEN the system SHALL continue processing other files

## Acceptance Criteria
- [ ] Storage directory location resolved correctly
- [ ] JSON files discovered and filtered appropriately
- [ ] Part files parsed into Rust structs
- [ ] Token metrics extracted correctly
- [ ] Aggregation calculates accurate totals
- [ ] Asynchronous processing implemented
- [ ] Caching mechanism functional
- [ ] Error handling comprehensive
- [ ] Unit tests for parsing logic
- [ ] Integration tests with sample data

## Technical Notes
- Use `std::fs` or `tokio::fs` for file operations
- Use `serde` and `serde_json` for JSON parsing
- Use `walkdir` or similar crate for directory traversal
- Consider using `rayon` for parallel file processing
- Store cache in memory or use lightweight file-based cache
- Define custom Error enum for specific failure modes

## Dependencies
- Feature 01: Project Setup & Boilerplate Applet
- Feature 02: Configuration & Authentication (for settings storage path if customizable)

## Future Enhancements
- Support for exporting usage data to JSONL format
- Historical trend analysis
- Per-session usage breakdown
- Model-specific usage tracking
- Cost estimation with configurable pricing
