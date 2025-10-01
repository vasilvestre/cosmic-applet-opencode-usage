// SPDX-License-Identifier: GPL-3.0-only

//! Data collection module for automatic usage snapshot management.
//!
//! This module provides business logic for when and how to collect usage snapshots.

use crate::core::database::{repository::UsageRepository, DatabaseManager};
use crate::core::opencode::UsageMetrics;
use chrono::NaiveDate;
use std::sync::{Arc, Mutex};

/// Error type for collector operations.
#[derive(Debug, thiserror::Error)]
pub enum CollectorError {
    /// Database error occurred
    #[error("Database error: {0}")]
    Database(#[from] crate::core::database::DatabaseError),

    /// Failed to acquire lock
    #[error("Failed to acquire lock")]
    LockError,
}

/// Manages data collection timing and logic.
pub struct DataCollector {
    repository: UsageRepository,
    last_collection: Arc<Mutex<Option<NaiveDate>>>,
}

impl DataCollector {
    /// Creates a new `DataCollector`.
    #[must_use]
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            repository: UsageRepository::new(db),
            last_collection: Arc::new(Mutex::new(None)),
        }
    }

    /// Checks if collection should happen based on date change.
    #[must_use]
    pub fn should_collect(&self) -> bool {
        let current_date = chrono::Utc::now().date_naive();

        let last = self.last_collection.lock().ok();
        if let Some(guard) = last {
            match *guard {
                None => true,                                 // First collection
                Some(last_date) => current_date != last_date, // Different day
            }
        } else {
            true // Lock error - try to collect
        }
    }

    /// Returns the date of the last collection, if any.
    #[must_use]
    pub fn get_last_collection_date(&self) -> Option<NaiveDate> {
        self.last_collection.lock().ok().and_then(|guard| *guard)
    }

    /// Collects and saves a usage snapshot if it hasn't been done today.
    ///
    /// Returns `Ok(true)` if snapshot was saved, `Ok(false)` if already collected today.
    ///
    /// # Errors
    ///
    /// Returns `CollectorError` if database operation fails or lock cannot be acquired.
    pub fn collect_and_save(&self, metrics: &UsageMetrics) -> Result<bool, CollectorError> {
        let current_date = chrono::Utc::now().date_naive();

        // Acquire lock
        let mut last_guard = self
            .last_collection
            .lock()
            .map_err(|_| CollectorError::LockError)?;

        // Check if we should collect
        let should_save = match *last_guard {
            None => true,
            Some(last_date) => current_date != last_date,
        };

        if should_save {
            // Save to database
            self.repository.save_snapshot(current_date, metrics)?;

            // Update last collection date
            *last_guard = Some(current_date);

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::DatabaseManager;
    use tempfile::TempDir;

    fn create_test_db() -> Arc<DatabaseManager> {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Arc::new(DatabaseManager::new_with_path(&db_path).unwrap())
    }

    #[test]
    fn test_collector_creation() {
        let db = create_test_db();
        let _collector = DataCollector::new(db);
        // If we get here, collector was created successfully
    }

    #[test]
    fn test_should_collect_first_time() {
        let db = create_test_db();
        let collector = DataCollector::new(db);

        assert!(collector.should_collect());
    }

    #[test]
    fn test_should_collect_same_day() {
        let db = create_test_db();
        let collector = DataCollector::new(db);

        // Simulate collection today
        let today = chrono::Utc::now().date_naive();
        {
            let mut last = collector.last_collection.lock().unwrap();
            *last = Some(today);
        }

        assert!(!collector.should_collect());
    }

    #[test]
    fn test_should_collect_next_day() {
        let db = create_test_db();
        let collector = DataCollector::new(db);

        // Simulate collection yesterday
        let yesterday = chrono::Utc::now().date_naive() - chrono::Duration::days(1);
        {
            let mut last = collector.last_collection.lock().unwrap();
            *last = Some(yesterday);
        }

        assert!(collector.should_collect());
    }

    #[test]
    fn test_get_last_collection_none() {
        let db = create_test_db();
        let collector = DataCollector::new(db);

        assert_eq!(collector.get_last_collection_date(), None);
    }

    #[test]
    fn test_get_last_collection_some() {
        let db = create_test_db();
        let collector = DataCollector::new(db);

        let test_date = chrono::Utc::now().date_naive();
        {
            let mut last = collector.last_collection.lock().unwrap();
            *last = Some(test_date);
        }

        assert_eq!(collector.get_last_collection_date(), Some(test_date));
    }

    #[test]
    fn test_collect_and_save_first_time() {
        use std::time::SystemTime;

        let db = create_test_db();
        let collector = DataCollector::new(Arc::clone(&db));

        let metrics = UsageMetrics {
            total_input_tokens: 100,
            total_output_tokens: 50,
            total_reasoning_tokens: 25,
            total_cache_write_tokens: 10,
            total_cache_read_tokens: 5,
            total_cost: 1.5,
            interaction_count: 1,
            timestamp: SystemTime::now(),
        };

        let result = collector.collect_and_save(&metrics);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Verify snapshot saved in database
        let today = chrono::Utc::now().date_naive();
        let snapshot = UsageRepository::new(db).get_snapshot(today).unwrap();
        assert!(snapshot.is_some());
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.input_tokens, 100);
        assert_eq!(snapshot.output_tokens, 50);
    }

    #[test]
    fn test_collect_and_save_same_day_twice() {
        use std::time::SystemTime;

        let db = create_test_db();
        let collector = DataCollector::new(Arc::clone(&db));

        let metrics = UsageMetrics {
            total_input_tokens: 100,
            total_output_tokens: 50,
            total_reasoning_tokens: 25,
            total_cache_write_tokens: 10,
            total_cache_read_tokens: 5,
            total_cost: 1.5,
            interaction_count: 1,
            timestamp: SystemTime::now(),
        };

        // First collection
        let result1 = collector.collect_and_save(&metrics);
        assert!(result1.is_ok());
        assert!(result1.unwrap());

        // Second collection same day
        let result2 = collector.collect_and_save(&metrics);
        assert!(result2.is_ok());
        assert!(!result2.unwrap());

        // Verify only one snapshot in database
        let today = chrono::Utc::now().date_naive();
        let repository = UsageRepository::new(db);
        let snapshot = repository.get_snapshot(today).unwrap();
        assert!(snapshot.is_some());
    }

    #[test]
    fn test_collect_and_save_different_days() {
        use std::time::SystemTime;

        let db = create_test_db();
        let collector = DataCollector::new(Arc::clone(&db));

        let metrics = UsageMetrics {
            total_input_tokens: 100,
            total_output_tokens: 50,
            total_reasoning_tokens: 25,
            total_cache_write_tokens: 10,
            total_cache_read_tokens: 5,
            total_cost: 1.5,
            interaction_count: 1,
            timestamp: SystemTime::now(),
        };

        // First collection
        let result1 = collector.collect_and_save(&metrics);
        assert!(result1.is_ok());
        assert!(result1.unwrap());

        // Simulate date change by resetting last_collection to yesterday
        let yesterday = chrono::Utc::now().date_naive() - chrono::Duration::days(1);
        {
            let mut last = collector.last_collection.lock().unwrap();
            *last = Some(yesterday);
        }

        // Second collection "next day"
        let result2 = collector.collect_and_save(&metrics);
        assert!(result2.is_ok());
        assert!(result2.unwrap());

        // Verify snapshots in database
        let repository = UsageRepository::new(db);
        let today = chrono::Utc::now().date_naive();
        let snapshot_today = repository.get_snapshot(today).unwrap();
        assert!(snapshot_today.is_some());
    }

    #[test]
    fn test_concurrent_collect() {
        use std::sync::Arc;
        use std::thread;
        use std::time::SystemTime;

        let db = create_test_db();
        let collector = Arc::new(DataCollector::new(Arc::clone(&db)));

        let metrics = UsageMetrics {
            total_input_tokens: 100,
            total_output_tokens: 50,
            total_reasoning_tokens: 25,
            total_cache_write_tokens: 10,
            total_cache_read_tokens: 5,
            total_cost: 1.5,
            interaction_count: 1,
            timestamp: SystemTime::now(),
        };

        // Spawn multiple threads trying to collect simultaneously
        let mut handles = vec![];
        for _ in 0..5 {
            let collector_clone = Arc::clone(&collector);
            let metrics_clone = metrics.clone();
            let handle = thread::spawn(move || collector_clone.collect_and_save(&metrics_clone));
            handles.push(handle);
        }

        // Collect results
        let mut results = vec![];
        for handle in handles {
            results.push(handle.join().unwrap());
        }

        // Exactly one thread should succeed (return Ok(true))
        let success_count = results
            .iter()
            .filter(|r| r.as_ref().ok() == Some(&true))
            .count();
        assert_eq!(success_count, 1);

        // Others should return Ok(false)
        let false_count = results
            .iter()
            .filter(|r| r.as_ref().ok() == Some(&false))
            .count();
        assert_eq!(false_count, 4);
    }
}
