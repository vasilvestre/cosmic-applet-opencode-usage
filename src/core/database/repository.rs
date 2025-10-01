// SPDX-License-Identifier: GPL-3.0-only

//! Repository layer for high-level database operations.
//!
//! This module provides a clean API for storing and retrieving usage snapshots.

use super::{DatabaseManager, Result};
use chrono::NaiveDate;
use rusqlite;
use std::sync::Arc;

/// A snapshot of usage metrics for a specific date.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSnapshot {
    pub date: NaiveDate,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_write_tokens: i64,
    pub cache_read_tokens: i64,
    pub total_cost: f64,
    pub interaction_count: i64,
}

/// High-level repository for usage snapshot operations.
pub struct UsageRepository {
    db: Arc<DatabaseManager>,
}

impl UsageRepository {
    /// Creates a new `UsageRepository`.
    #[must_use]
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self { db }
    }

    /// Saves a usage snapshot for a specific date.
    ///
    /// If a snapshot already exists for this date, it will be replaced.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub fn save_snapshot(
        &self,
        date: NaiveDate,
        metrics: &crate::core::opencode::UsageMetrics,
    ) -> Result<()> {
        let conn = self.db.get_connection();

        conn.execute(
            "INSERT OR REPLACE INTO usage_snapshots 
             (date, input_tokens, output_tokens, reasoning_tokens, cache_write_tokens, cache_read_tokens, total_cost, interaction_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                date.to_string(),
                i64::try_from(metrics.total_input_tokens).unwrap_or(0),
                i64::try_from(metrics.total_output_tokens).unwrap_or(0),
                i64::try_from(metrics.total_reasoning_tokens).unwrap_or(0),
                i64::try_from(metrics.total_cache_write_tokens).unwrap_or(0),
                i64::try_from(metrics.total_cache_read_tokens).unwrap_or(0),
                metrics.total_cost,
                i64::try_from(metrics.interaction_count).unwrap_or(0),
                chrono::Utc::now().to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// Retrieves a usage snapshot for a specific date.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub fn get_snapshot(&self, date: NaiveDate) -> Result<Option<UsageSnapshot>> {
        let conn = self.db.get_connection();

        let mut stmt = conn.prepare(
            "SELECT date, input_tokens, output_tokens, reasoning_tokens, cache_write_tokens, cache_read_tokens, total_cost, interaction_count
             FROM usage_snapshots
             WHERE date = ?1"
        )?;

        let result = stmt.query_row(rusqlite::params![date.to_string()], |row| {
            Self::row_to_snapshot(row)
        });

        match result {
            Ok(snapshot) => Ok(Some(snapshot)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Retrieves usage snapshots within a date range (inclusive).
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub fn get_range(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<UsageSnapshot>> {
        let conn = self.db.get_connection();

        let mut stmt = conn.prepare(
            "SELECT date, input_tokens, output_tokens, reasoning_tokens, cache_write_tokens, cache_read_tokens, total_cost, interaction_count
             FROM usage_snapshots
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date ASC"
        )?;

        let snapshots = stmt
            .query_map(
                rusqlite::params![start.to_string(), end.to_string()],
                Self::row_to_snapshot,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(snapshots)
    }

    /// Retrieves the most recent usage snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub fn get_latest(&self) -> Result<Option<UsageSnapshot>> {
        let conn = self.db.get_connection();

        let mut stmt = conn.prepare(
            "SELECT date, input_tokens, output_tokens, reasoning_tokens, cache_write_tokens, cache_read_tokens, total_cost, interaction_count
             FROM usage_snapshots
             ORDER BY date DESC
             LIMIT 1"
        )?;

        let result = stmt.query_row([], Self::row_to_snapshot);

        match result {
            Ok(snapshot) => Ok(Some(snapshot)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Deletes snapshots older than the specified number of days.
    ///
    /// Returns the number of snapshots deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub fn delete_old(&self, days: u32) -> Result<usize> {
        let conn = self.db.get_connection();

        let cutoff_date = chrono::Utc::now().date_naive() - chrono::Duration::days(i64::from(days));

        let deleted = conn.execute(
            "DELETE FROM usage_snapshots WHERE date < ?1",
            rusqlite::params![cutoff_date.to_string()],
        )?;

        Ok(deleted)
    }

    /// Aggregates usage data for a week into a single summary.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub fn get_week_summary(&self, week_start: NaiveDate) -> Result<WeekSummary> {
        let week_end = week_start + chrono::Duration::days(6);
        let snapshots = self.get_range(week_start, week_end)?;

        let summary = WeekSummary {
            start_date: week_start,
            end_date: week_end,
            total_input_tokens: snapshots.iter().map(|s| s.input_tokens).sum(),
            total_output_tokens: snapshots.iter().map(|s| s.output_tokens).sum(),
            total_reasoning_tokens: snapshots.iter().map(|s| s.reasoning_tokens).sum(),
            total_cache_write_tokens: snapshots.iter().map(|s| s.cache_write_tokens).sum(),
            total_cache_read_tokens: snapshots.iter().map(|s| s.cache_read_tokens).sum(),
            total_cost: snapshots.iter().map(|s| s.total_cost).sum(),
            total_interactions: snapshots.iter().map(|s| s.interaction_count).sum(),
        };

        Ok(summary)
    }

    /// Helper to convert a database row to a `UsageSnapshot`.
    fn row_to_snapshot(row: &rusqlite::Row) -> std::result::Result<UsageSnapshot, rusqlite::Error> {
        let date_str: String = row.get(0)?;
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        Ok(UsageSnapshot {
            date,
            input_tokens: row.get(1)?,
            output_tokens: row.get(2)?,
            reasoning_tokens: row.get(3)?,
            cache_write_tokens: row.get(4)?,
            cache_read_tokens: row.get(5)?,
            total_cost: row.get(6)?,
            interaction_count: row.get(7)?,
        })
    }
}

/// A weekly summary of aggregated usage metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct WeekSummary {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_reasoning_tokens: i64,
    pub total_cache_write_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cost: f64,
    pub total_interactions: i64,
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // Tests use exact float comparisons for simplicity
mod tests {
    use super::*;
    use crate::core::opencode::UsageMetrics;
    use std::time::SystemTime;
    use tempfile::TempDir;

    fn create_test_db() -> Arc<DatabaseManager> {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Arc::new(DatabaseManager::new_with_path(&db_path).unwrap())
    }

    fn create_test_metrics() -> UsageMetrics {
        UsageMetrics {
            total_input_tokens: 600,
            total_output_tokens: 400,
            total_reasoning_tokens: 50,
            total_cache_write_tokens: 100,
            total_cache_read_tokens: 200,
            total_cost: 0.15,
            interaction_count: 5,
            timestamp: SystemTime::now(),
        }
    }

    #[test]
    fn test_usage_snapshot_creation() {
        let snapshot = UsageSnapshot {
            date: NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
            input_tokens: 600,
            output_tokens: 400,
            reasoning_tokens: 50,
            cache_write_tokens: 100,
            cache_read_tokens: 200,
            total_cost: 0.15,
            interaction_count: 5,
        };

        assert_eq!(snapshot.input_tokens, 600);
        assert_eq!(snapshot.output_tokens, 400);
        assert_eq!(snapshot.reasoning_tokens, 50);
        assert_eq!(snapshot.cache_write_tokens, 100);
        assert_eq!(snapshot.cache_read_tokens, 200);
        assert_eq!(snapshot.total_cost, 0.15);
        assert_eq!(snapshot.interaction_count, 5);
    }

    #[test]
    fn test_repository_creation() {
        let db = create_test_db();
        let _repository = UsageRepository::new(db);
        // If we get here, repository was created successfully
    }

    #[test]
    fn test_save_snapshot() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);
        let date = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
        let metrics = create_test_metrics();

        let result = repository.save_snapshot(date, &metrics);
        if let Err(e) = &result {
            eprintln!("Error saving snapshot: {e:?}");
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_save_snapshot_duplicate_date() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);
        let date = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();

        let metrics1 = UsageMetrics {
            total_input_tokens: 600,
            total_output_tokens: 400,
            total_reasoning_tokens: 50,
            total_cache_write_tokens: 100,
            total_cache_read_tokens: 200,
            total_cost: 0.15,
            interaction_count: 5,
            timestamp: SystemTime::now(),
        };

        let metrics2 = UsageMetrics {
            total_input_tokens: 800,
            total_output_tokens: 500,
            total_reasoning_tokens: 60,
            total_cache_write_tokens: 150,
            total_cache_read_tokens: 250,
            total_cost: 0.25,
            interaction_count: 8,
            timestamp: SystemTime::now(),
        };

        // First save
        let result1 = repository.save_snapshot(date, &metrics1);
        assert!(result1.is_ok());

        // Second save with same date should succeed (REPLACE behavior)
        let result2 = repository.save_snapshot(date, &metrics2);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_get_snapshot_exists() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);
        let date = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
        let metrics = create_test_metrics();

        // Save first
        repository.save_snapshot(date, &metrics).unwrap();

        // Then retrieve
        let result = repository.get_snapshot(date);
        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert!(snapshot.is_some());

        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.date, date);
        assert_eq!(snapshot.input_tokens, 600);
        assert_eq!(snapshot.output_tokens, 400);
        assert_eq!(snapshot.reasoning_tokens, 50);
        assert_eq!(snapshot.cache_write_tokens, 100);
        assert_eq!(snapshot.cache_read_tokens, 200);
        assert_eq!(snapshot.total_cost, 0.15);
        assert_eq!(snapshot.interaction_count, 5);
    }

    #[test]
    fn test_get_snapshot_missing() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);
        let date = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();

        let result = repository.get_snapshot(date);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_get_range_multiple() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);

        // Save 5 snapshots across different dates
        for i in 1..=5 {
            let date = NaiveDate::from_ymd_opt(2025, 10, i).unwrap();
            let metrics = create_test_metrics();
            repository.save_snapshot(date, &metrics).unwrap();
        }

        // Get middle 3 (Oct 2-4)
        let start = NaiveDate::from_ymd_opt(2025, 10, 2).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 10, 4).unwrap();

        let result = repository.get_range(start, end);
        assert!(result.is_ok());

        let snapshots = result.unwrap();
        assert_eq!(snapshots.len(), 3);
        assert_eq!(
            snapshots[0].date,
            NaiveDate::from_ymd_opt(2025, 10, 2).unwrap()
        );
        assert_eq!(
            snapshots[1].date,
            NaiveDate::from_ymd_opt(2025, 10, 3).unwrap()
        );
        assert_eq!(
            snapshots[2].date,
            NaiveDate::from_ymd_opt(2025, 10, 4).unwrap()
        );
    }

    #[test]
    fn test_get_range_empty() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);

        let start = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 10, 5).unwrap();

        let result = repository.get_range(start, end);
        assert!(result.is_ok());

        let snapshots = result.unwrap();
        assert_eq!(snapshots.len(), 0);
    }

    #[test]
    fn test_get_latest_exists() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);

        // Save 3 snapshots with different dates
        let dates = vec![
            NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 10, 5).unwrap(),
            NaiveDate::from_ymd_opt(2025, 10, 3).unwrap(),
        ];

        for date in &dates {
            let metrics = create_test_metrics();
            repository.save_snapshot(*date, &metrics).unwrap();
        }

        let result = repository.get_latest();
        assert!(result.is_ok());

        let snapshot = result.unwrap();
        assert!(snapshot.is_some());

        // Should return Oct 5 (most recent)
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.date, NaiveDate::from_ymd_opt(2025, 10, 5).unwrap());
    }

    #[test]
    fn test_get_latest_empty() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);

        let result = repository.get_latest();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_delete_old_removes_old() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);

        let today = chrono::Utc::now().date_naive();

        // Save snapshots for 10 days ago, 5 days ago, and today
        let dates = vec![
            today - chrono::Duration::days(10),
            today - chrono::Duration::days(5),
            today,
        ];

        for date in &dates {
            let metrics = create_test_metrics();
            repository.save_snapshot(*date, &metrics).unwrap();
        }

        // Delete older than 7 days
        let result = repository.delete_old(7);
        assert!(result.is_ok());

        let deleted_count = result.unwrap();
        assert_eq!(deleted_count, 1); // Only 10-days-ago should be deleted

        // Verify remaining snapshots
        let all = repository
            .get_range(
                today - chrono::Duration::days(20),
                today + chrono::Duration::days(1),
            )
            .unwrap();

        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_delete_old_empty_database() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);

        let result = repository.delete_old(7);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_week_summary_aggregates_correctly() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);

        // Create a week of data (7 days)
        let week_start = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(); // Oct 1-7

        for day_offset in 0..7 {
            let date = week_start + chrono::Duration::days(day_offset);
            let day_num = u64::try_from(day_offset + 1).unwrap();
            let metrics = UsageMetrics {
                total_input_tokens: 1000 * day_num, // 1000, 2000, 3000, etc.
                total_output_tokens: 500 * day_num,
                total_reasoning_tokens: 100 * day_num,
                total_cache_write_tokens: 50 * day_num,
                total_cache_read_tokens: 200 * day_num,
                total_cost: 0.10 * day_num as f64,
                interaction_count: 10 * (day_num as usize),
                timestamp: SystemTime::now(),
            };
            repository.save_snapshot(date, &metrics).unwrap();
        }

        // Get week summary
        let summary = repository.get_week_summary(week_start).unwrap();

        // Verify aggregation
        assert_eq!(summary.start_date, week_start);
        assert_eq!(summary.end_date, week_start + chrono::Duration::days(6));

        // Sum of 1000, 2000, 3000, 4000, 5000, 6000, 7000 = 28000
        assert_eq!(summary.total_input_tokens, 28000);
        // Sum of 500, 1000, 1500, 2000, 2500, 3000, 3500 = 14000
        assert_eq!(summary.total_output_tokens, 14000);
        // Sum of 100, 200, 300, 400, 500, 600, 700 = 2800
        assert_eq!(summary.total_reasoning_tokens, 2800);
        // Sum of 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70 = 2.80
        assert!((summary.total_cost - 2.80).abs() < 0.01);
        // Sum of 10, 20, 30, 40, 50, 60, 70 = 280
        assert_eq!(summary.total_interactions, 280);
    }

    #[test]
    fn test_week_summary_empty_week() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);

        let week_start = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();

        // Get summary for week with no data
        let summary = repository.get_week_summary(week_start).unwrap();

        // Should return zeros for empty week
        assert_eq!(summary.total_input_tokens, 0);
        assert_eq!(summary.total_output_tokens, 0);
        assert_eq!(summary.total_reasoning_tokens, 0);
        assert_eq!(summary.total_cost, 0.0);
        assert_eq!(summary.total_interactions, 0);
    }

    #[test]
    fn test_week_summary_partial_week() {
        let db = create_test_db();
        let repository = UsageRepository::new(db);

        let week_start = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();

        // Only add data for 3 days
        for day_offset in 0..3 {
            let date = week_start + chrono::Duration::days(day_offset);
            let metrics = create_test_metrics();
            repository.save_snapshot(date, &metrics).unwrap();
        }

        let summary = repository.get_week_summary(week_start).unwrap();

        // Should aggregate only the 3 days (3 * 600 = 1800)
        assert_eq!(summary.total_input_tokens, 1800);
        assert_eq!(summary.total_output_tokens, 1200);
    }
}
