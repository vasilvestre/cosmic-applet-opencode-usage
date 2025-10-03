# Database Integration Requirements

## Overview
This feature provides persistent storage for OpenCode usage metrics using SQLite, enabling historical tracking and trend analysis.

## Functional Requirements

### FR1: Database Initialization
**FR1.1** The system SHALL create a SQLite database at `~/.local/share/cosmic-applet-opencode-usage/usage.db` if it does not exist.

**FR1.2** WHEN the database is created THEN the system SHALL apply all necessary schema migrations.

**FR1.3** WHEN the parent directory does not exist THEN the system SHALL create it before initializing the database.

### FR2: Schema Management
**FR2.1** The system SHALL maintain a `usage_snapshots` table with the following columns:
- id (INTEGER PRIMARY KEY AUTOINCREMENT)
- date (TEXT, ISO 8601 format)
- input_tokens (INTEGER NOT NULL)
- output_tokens (INTEGER NOT NULL)
- reasoning_tokens (INTEGER NOT NULL)
- cache_write_tokens (INTEGER NOT NULL)
- cache_read_tokens (INTEGER NOT NULL)
- total_cost (REAL NOT NULL)
- interaction_count (INTEGER NOT NULL)
- created_at (TEXT, ISO 8601 timestamp)

**FR2.2** The system SHALL maintain a `schema_version` table to track migration history.

**FR2.3** WHEN the application starts THEN the system SHALL apply any pending migrations.

### FR3: Connection Management
**FR3.1** The system SHALL provide a connection pool or managed connection to the database.

**FR3.2** WHEN a database operation fails THEN the system SHALL return a descriptive error.

**FR3.3** The system SHALL handle concurrent access safely using SQLite's built-in locking mechanisms.

### FR4: Error Handling
**FR4.1** WHEN the database cannot be created THEN the system SHALL return an error indicating the failure reason.

**FR4.2** WHEN a migration fails THEN the system SHALL rollback the transaction and return an error.

**FR4.3** WHEN the database file is corrupted THEN the system SHALL return an appropriate error message.

## Non-Functional Requirements

### NFR1: Performance
**NFR1.1** Database initialization SHALL complete within 500ms.

**NFR1.2** Database operations SHALL not block the UI thread.

### NFR2: Reliability
**NFR2.1** The system SHALL use WAL (Write-Ahead Logging) mode for improved concurrency.

**NFR2.2** All database operations SHALL be atomic and consistent.

### NFR3: Maintainability
**NFR3.1** The system SHALL support forward-compatible schema migrations.

**NFR3.2** Database code SHALL follow Rust error handling best practices using `Result<T, E>`.

### NFR4: Security
**NFR4.1** The database file SHALL have appropriate permissions (user read/write only).

## Acceptance Criteria
- [ ] Database is created automatically on first run
- [ ] All schema tables are created correctly
- [ ] Migrations can be applied incrementally
- [ ] Database operations return proper error types
- [ ] All tests pass with >90% coverage
- [ ] No clippy warnings in database code
