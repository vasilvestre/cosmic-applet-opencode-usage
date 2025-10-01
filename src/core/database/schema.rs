// SPDX-License-Identifier: GPL-3.0-only

//! SQL schema definitions for the usage tracking database.
//!
//! This module contains all SQL statements for creating tables, indexes,
//! and other database objects.

/// SQL statement to create the `usage_snapshots` table.
///
/// This table stores daily snapshots of `OpenCode` usage metrics.
pub const CREATE_USAGE_SNAPSHOTS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS usage_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    reasoning_tokens INTEGER NOT NULL,
    cache_write_tokens INTEGER NOT NULL,
    cache_read_tokens INTEGER NOT NULL,
    total_cost REAL NOT NULL,
    interaction_count INTEGER NOT NULL,
    created_at TEXT NOT NULL
)
";

/// SQL statement to create the `schema_version` table.
///
/// This table tracks which migrations have been applied to the database.
pub const CREATE_SCHEMA_VERSION_TABLE: &str = "
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
)
";

/// SQL statement to create an index on the `date` column of `usage_snapshots`.
///
/// This index improves query performance when filtering or sorting by date.
pub const CREATE_DATE_INDEX: &str = "
CREATE INDEX IF NOT EXISTS idx_usage_snapshots_date
ON usage_snapshots(date)
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_snapshots_table_sql() {
        assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("usage_snapshots"));
        assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("input_tokens"));
        assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("output_tokens"));
        assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("reasoning_tokens"));
        assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("cache_write_tokens"));
        assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("cache_read_tokens"));
        assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("total_cost"));
        assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("interaction_count"));
        assert!(CREATE_USAGE_SNAPSHOTS_TABLE.contains("created_at"));
    }

    #[test]
    fn test_schema_version_table_sql() {
        assert!(CREATE_SCHEMA_VERSION_TABLE.contains("schema_version"));
        assert!(CREATE_SCHEMA_VERSION_TABLE.contains("version"));
        assert!(CREATE_SCHEMA_VERSION_TABLE.contains("applied_at"));
    }

    #[test]
    fn test_date_index_sql() {
        assert!(CREATE_DATE_INDEX.contains("idx_usage_snapshots_date"));
        assert!(CREATE_DATE_INDEX.contains("usage_snapshots"));
        assert!(CREATE_DATE_INDEX.contains("date"));
    }
}
