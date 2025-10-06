use crate::core::opencode::{
    FileMetadata, ScannerError, StorageScanner, UsageAggregator, UsageMetrics, UsageParser,
    UsagePart,
};
use chrono::{Datelike, Local, TimeZone};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use thiserror::Error;

/// Cache duration: 5 minutes
const CACHE_DURATION: Duration = Duration::from_secs(5 * 60);

/// Error types for reader operations
#[derive(Debug, Error)]
pub enum ReaderError {
    #[error("Scanner error: {0}")]
    ScannerError(#[from] ScannerError),

    #[error("No usage data found")]
    NoDataFound,

    #[error("Failed to access storage: {0}")]
    AccessError(String),
}

/// Cached parsed file data
#[derive(Debug, Clone)]
struct CachedFile {
    part: UsagePart,
    modified: SystemTime,
}

/// Cached usage data with incremental file tracking
#[derive(Debug, Clone)]
struct CachedData {
    metrics: UsageMetrics,
    timestamp: SystemTime,
    /// Map of file path to cached parsed data
    files: HashMap<PathBuf, CachedFile>,
}

/// Main orchestrator for reading `OpenCode` usage data
pub struct OpenCodeUsageReader {
    scanner: StorageScanner,
    cache: Option<CachedData>,
}

impl OpenCodeUsageReader {
    /// Create a new reader with default `OpenCode` storage path
    ///
    /// # Errors
    /// Returns an error if the scanner cannot be initialized.
    pub fn new() -> Result<Self, ReaderError> {
        let scanner = StorageScanner::new()?;
        Ok(Self {
            scanner,
            cache: None,
        })
    }

    /// Create a reader with a custom path (useful for testing)
    ///
    /// # Errors
    /// Returns an error if the scanner cannot be initialized with the given path.
    pub fn new_with_path(path: &str) -> Result<Self, ReaderError> {
        let scanner = StorageScanner::with_path(std::path::PathBuf::from(path))?;
        Ok(Self {
            scanner,
            cache: None,
        })
    }

    /// Create a reader with a custom scanner (useful for testing)
    #[must_use]
    pub fn with_scanner(scanner: StorageScanner) -> Self {
        Self {
            scanner,
            cache: None,
        }
    }

    /// Get the storage path
    #[must_use]
    pub fn storage_path(&self) -> &PathBuf {
        self.scanner.storage_path()
    }

    /// Get usage metrics, using cache if available and not expired
    ///
    /// # Errors
    /// Returns an error if no data is found or if parsing fails.
    pub fn get_usage(&mut self) -> Result<UsageMetrics, ReaderError> {
        // Check if we have valid cached data (time-based)
        if let Some(cached) = &self.cache {
            if !self.should_refresh_cache() {
                return Ok(cached.metrics.clone());
            }
        }

        // Scan files with metadata
        let files = self.scanner.scan_with_metadata()?;

        if files.is_empty() {
            return Err(ReaderError::NoDataFound);
        }

        // Determine which files need to be parsed
        let (parts_to_aggregate, new_file_cache) = self.incremental_parse(&files)?;

        if parts_to_aggregate.is_empty() {
            return Err(ReaderError::NoDataFound);
        }

        // Aggregate all parts
        let mut aggregator = UsageAggregator::new();
        for part in parts_to_aggregate {
            aggregator.add_part(&part);
        }
        let metrics = aggregator.finalize();

        // Update cache
        self.cache = Some(CachedData {
            metrics: metrics.clone(),
            timestamp: metrics.timestamp,
            files: new_file_cache,
        });

        Ok(metrics)
    }

    /// Get usage metrics for today only (files modified today)
    ///
    /// # Errors
    /// Returns an error if no data is found for today or if parsing fails.
    pub fn get_usage_today(&mut self) -> Result<UsageMetrics, ReaderError> {
        // Calculate start of today (midnight) as cutoff time
        let cutoff = Self::get_today_start();

        // Scan only files modified since start of today
        let today_files = self.scanner.scan_modified_since(cutoff)?;

        if today_files.is_empty() {
            return Err(ReaderError::NoDataFound);
        }

        // Parse and aggregate filtered files
        self.parse_and_aggregate(&today_files)
    }

    /// Get usage metrics for this month only (files modified this month)
    ///
    /// # Errors
    /// Returns an error if no data is found for this month or if parsing fails.
    pub fn get_usage_month(&mut self) -> Result<UsageMetrics, ReaderError> {
        // Calculate start of month (first day at midnight) as cutoff time
        let cutoff = Self::get_month_start();

        // Scan only files modified since start of month
        let month_files = self.scanner.scan_modified_since(cutoff)?;

        if month_files.is_empty() {
            return Err(ReaderError::NoDataFound);
        }

        // Parse and aggregate filtered files
        self.parse_and_aggregate(&month_files)
    }

    /// Get usage metrics for last month only (files modified during last month)
    ///
    /// # Errors
    /// Returns an error if no data is found for last month or if parsing fails.
    pub fn get_usage_last_month(&mut self) -> Result<UsageMetrics, ReaderError> {
        // Calculate start of last month and start of current month
        let last_month_start = Self::get_last_month_start();
        let this_month_start = Self::get_month_start();

        // Scan only files modified since start of last month
        let last_month_files = self.scanner.scan_modified_since(last_month_start)?;

        if last_month_files.is_empty() {
            return Err(ReaderError::NoDataFound);
        }

        // Filter to only files from last month (before this month started)
        let last_month_only: Vec<_> = last_month_files
            .into_iter()
            .filter(|file| file.modified < this_month_start)
            .collect();

        if last_month_only.is_empty() {
            return Err(ReaderError::NoDataFound);
        }

        // Parse and aggregate filtered files
        self.parse_and_aggregate(&last_month_only)
    }

    /// Get the start of today (midnight) as `SystemTime`
    fn get_today_start() -> SystemTime {
        let now = SystemTime::now();
        let now_since_epoch = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        let now_secs = now_since_epoch.as_secs();

        // Calculate start of today (midnight) in seconds since epoch
        // 86400 seconds = 24 hours
        let today_start_secs = (now_secs / 86400) * 86400;

        SystemTime::UNIX_EPOCH + Duration::from_secs(today_start_secs)
    }

    /// Get the start of this month (first day at midnight) as `SystemTime`
    fn get_month_start() -> SystemTime {
        use std::time::UNIX_EPOCH;

        // Get current date in local timezone
        let now = Local::now();

        // Create a DateTime for the first day of the current month at midnight
        let month_start = Local
            .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
            .single()
            .expect("Should create valid date for first day of month");

        // Convert to SystemTime using the timestamp
        // timestamp() returns seconds since UNIX_EPOCH in UTC
        let timestamp = month_start.timestamp();
        // Ensure timestamp is non-negative before casting
        #[allow(clippy::cast_sign_loss)]
        let timestamp_u64 = timestamp.max(0) as u64;
        UNIX_EPOCH + Duration::from_secs(timestamp_u64)
    }

    /// Get the start of last month (first day at midnight) as `SystemTime`
    fn get_last_month_start() -> SystemTime {
        use std::time::UNIX_EPOCH;

        // Get current date in local timezone
        let now = Local::now();

        // Calculate last month's year and month
        let (last_month_year, last_month) = if now.month() == 1 {
            // If current month is January, last month is December of previous year
            (now.year() - 1, 12)
        } else {
            (now.year(), now.month() - 1)
        };

        // Create a DateTime for the first day of last month at midnight
        let last_month_start = Local
            .with_ymd_and_hms(last_month_year, last_month, 1, 0, 0, 0)
            .single()
            .expect("Should create valid date for first day of last month");

        // Convert to SystemTime using the timestamp
        let timestamp = last_month_start.timestamp();
        // Ensure timestamp is non-negative before casting
        #[allow(clippy::cast_sign_loss)]
        let timestamp_u64 = timestamp.max(0) as u64;
        UNIX_EPOCH + Duration::from_secs(timestamp_u64)
    }

    /// Parse and aggregate usage files (shared logic for `get_usage` and `get_usage_today`)
    fn parse_and_aggregate(&mut self, files: &[FileMetadata]) -> Result<UsageMetrics, ReaderError> {
        // Determine which files need to be parsed
        let (parts_to_aggregate, _) = self.incremental_parse(files)?;

        if parts_to_aggregate.is_empty() {
            return Err(ReaderError::NoDataFound);
        }

        // Aggregate all parts
        let mut aggregator = UsageAggregator::new();
        for part in parts_to_aggregate {
            aggregator.add_part(&part);
        }
        let metrics = aggregator.finalize();

        Ok(metrics)
    }

    /// Parse only new or modified files, reusing cached results for unchanged files
    #[allow(clippy::unnecessary_wraps)] // May return errors in future implementations
    fn incremental_parse(
        &self,
        files: &[FileMetadata],
    ) -> Result<(Vec<UsagePart>, HashMap<PathBuf, CachedFile>), ReaderError> {
        let mut parts = Vec::new();
        let mut new_cache = HashMap::new();

        for file_meta in files {
            // Check if we have a cached version of this file
            let needs_parse = if let Some(cached) = &self.cache {
                if let Some(cached_file) = cached.files.get(&file_meta.path) {
                    // File exists in cache - check if modified
                    if cached_file.modified == file_meta.modified {
                        // File unchanged - reuse cached result
                        parts.push(cached_file.part.clone());
                        new_cache.insert(file_meta.path.clone(), cached_file.clone());
                        false
                    } else {
                        // File modified - needs re-parse
                        true
                    }
                } else {
                    // New file not in cache - needs parse
                    true
                }
            } else {
                // No cache - needs parse
                true
            };

            if needs_parse {
                // Parse the file
                if let Ok(Some(part)) = UsageParser::parse_file(&file_meta.path) {
                    parts.push(part.clone());
                    new_cache.insert(
                        file_meta.path.clone(),
                        CachedFile {
                            part,
                            modified: file_meta.modified,
                        },
                    );
                } else {
                    // File parsed but no tokens, or invalid JSON - skip silently
                }
            }
        }

        Ok((parts, new_cache))
    }

    /// Check if cache should be refreshed
    fn should_refresh_cache(&self) -> bool {
        if let Some(cached) = &self.cache {
            if let Ok(elapsed) = cached.timestamp.elapsed() {
                return elapsed >= CACHE_DURATION;
            }
        }
        true
    }
}

#[cfg(test)]
#[allow(clippy::cast_possible_wrap)] // Tests use time conversions
#[allow(clippy::cast_sign_loss)] // Tests use time conversions
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    /// Helper to create a temporary test directory
    fn create_test_dir(name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("opencode_reader_test_{name}"));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("Failed to create test directory");
        temp_dir
    }

    /// Helper to create a usage part JSON file
    fn create_usage_file(dir: &Path, name: &str, input: u64, output: u64, cost: f64) {
        let content = format!(
            r#"{{
                "id": "prt_{name}",
                "messageID": "msg_test",
                "sessionID": "ses_test",
                "type": "step-finish",
                "tokens": {{
                    "input": {input},
                    "output": {output},
                    "reasoning": 0,
                    "cache": {{
                        "write": 0,
                        "read": 0
                    }}
                }},
                "cost": {cost}
            }}"#
        );

        let file_path = dir.join(format!("{name}.json"));
        let mut file = fs::File::create(file_path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test file");
    }

    // Test 1: Read and aggregate sample data
    #[test]
    fn test_reader_with_sample_data() {
        let test_dir = create_test_dir("sample_data");

        // Create sample usage files
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);
        create_usage_file(&test_dir, "file2", 200, 100, 0.50);
        create_usage_file(&test_dir, "file3", 150, 75, 0.30);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        let metrics = reader.get_usage().expect("Should read usage data");

        assert_eq!(metrics.total_input_tokens, 450);
        assert_eq!(metrics.total_output_tokens, 225);
        assert_eq!(metrics.interaction_count, 3);
        assert!((metrics.total_cost - 1.05).abs() < 0.001);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 2: Handle directory with no usage files
    #[test]
    fn test_reader_with_no_data() {
        let test_dir = create_test_dir("no_data");

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        let result = reader.get_usage();

        // Should return error when no data found
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ReaderError::NoDataFound));

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 3: Verify cache is used on second call
    #[test]
    fn test_reader_caching() {
        let test_dir = create_test_dir("caching");

        create_usage_file(&test_dir, "file1", 100, 50, 0.25);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // First call - should scan
        let metrics1 = reader.get_usage().expect("Should read usage data");
        let timestamp1 = metrics1.timestamp;

        // Second call immediately - should use cache
        let metrics2 = reader
            .get_usage()
            .expect("Should read usage data from cache");
        let timestamp2 = metrics2.timestamp;

        // Timestamps should be identical (from cache)
        assert_eq!(timestamp1, timestamp2, "Should use cached data");
        assert_eq!(metrics1.total_input_tokens, metrics2.total_input_tokens);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 4: Verify cache expires after 5 minutes
    #[test]
    fn test_reader_cache_expiry() {
        let test_dir = create_test_dir("cache_expiry");

        create_usage_file(&test_dir, "file1", 100, 50, 0.25);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // Manually set cache with old timestamp
        let old_metrics = UsageMetrics {
            total_input_tokens: 999,
            total_output_tokens: 999,
            total_reasoning_tokens: 0,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 9.99,
            interaction_count: 1,
            timestamp: SystemTime::now() - Duration::from_secs(6 * 60), // 6 minutes ago
        };

        reader.cache = Some(CachedData {
            metrics: old_metrics.clone(),
            timestamp: old_metrics.timestamp,
            files: HashMap::new(),
        });

        // Should detect expired cache and refresh
        let metrics = reader.get_usage().expect("Should refresh expired cache");

        // Should have new data, not cached data
        assert_eq!(
            metrics.total_input_tokens, 100,
            "Should not use expired cache"
        );
        assert_ne!(metrics.total_input_tokens, 999);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 5: Continue despite invalid JSON files
    #[test]
    fn test_reader_skips_invalid_files() {
        let test_dir = create_test_dir("invalid_files");

        // Create valid files
        create_usage_file(&test_dir, "valid1", 100, 50, 0.25);
        create_usage_file(&test_dir, "valid2", 200, 100, 0.50);

        // Create invalid JSON file
        let invalid_path = test_dir.join("invalid.json");
        let mut file = fs::File::create(invalid_path).expect("Should create file");
        file.write_all(b"{ invalid json !!!").expect("Should write");
        drop(file);

        // Create file without tokens (should be skipped)
        let no_tokens_path = test_dir.join("no_tokens.json");
        let mut file = fs::File::create(no_tokens_path).expect("Should create file");
        file.write_all(
            br#"{
            "id": "prt_test",
            "messageID": "msg_test",
            "sessionID": "ses_test",
            "type": "step-start",
            "cost": 0
        }"#,
        )
        .expect("Should write");
        drop(file);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        let metrics = reader
            .get_usage()
            .expect("Should read valid files despite invalid ones");

        // Should only aggregate the 2 valid files
        assert_eq!(metrics.total_input_tokens, 300);
        assert_eq!(metrics.total_output_tokens, 150);
        assert_eq!(metrics.interaction_count, 2);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 6: Incremental parsing - only parse new files
    #[test]
    fn test_reader_incremental_parsing_new_files() {
        let test_dir = create_test_dir("incremental_new");

        // Create initial files
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);
        create_usage_file(&test_dir, "file2", 200, 100, 0.50);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // First read - should parse all files
        let metrics1 = reader.get_usage().expect("Should read initial files");
        assert_eq!(metrics1.total_input_tokens, 300);
        assert_eq!(metrics1.interaction_count, 2);

        // Add a new file
        std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamp
        create_usage_file(&test_dir, "file3", 150, 75, 0.30);

        // Force cache expiry
        if let Some(ref mut cache) = reader.cache {
            cache.timestamp = SystemTime::now() - Duration::from_secs(6 * 60);
        }

        // Second read - should parse new file and reuse cached results for old files
        let metrics2 = reader.get_usage().expect("Should read with new file");
        assert_eq!(metrics2.total_input_tokens, 450);
        assert_eq!(metrics2.interaction_count, 3);

        // Verify cache contains all 3 files
        assert_eq!(reader.cache.as_ref().unwrap().files.len(), 3);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 7: Incremental parsing - re-parse modified files
    #[test]
    fn test_reader_incremental_parsing_modified_files() {
        let test_dir = create_test_dir("incremental_modified");

        // Create initial file
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // First read
        let metrics1 = reader.get_usage().expect("Should read initial file");
        assert_eq!(metrics1.total_input_tokens, 100);

        // Modify the file (overwrite with new content)
        std::thread::sleep(std::time::Duration::from_millis(50)); // Ensure different timestamp
        create_usage_file(&test_dir, "file1", 500, 250, 1.00);

        // Force cache expiry
        if let Some(ref mut cache) = reader.cache {
            cache.timestamp = SystemTime::now() - Duration::from_secs(6 * 60);
        }

        // Second read - should detect modification and re-parse
        let metrics2 = reader.get_usage().expect("Should read modified file");
        assert_eq!(
            metrics2.total_input_tokens, 500,
            "Should have updated values from modified file"
        );
        assert_eq!(metrics2.total_output_tokens, 250);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 8: Incremental parsing - reuse cache for unchanged files
    #[test]
    fn test_reader_incremental_parsing_unchanged_files() {
        let test_dir = create_test_dir("incremental_unchanged");

        // Create files
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);
        create_usage_file(&test_dir, "file2", 200, 100, 0.50);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // First read
        let metrics1 = reader.get_usage().expect("Should read files");
        assert_eq!(metrics1.total_input_tokens, 300);

        // Get file paths from cache
        let cached_files_count = reader.cache.as_ref().unwrap().files.len();
        assert_eq!(cached_files_count, 2);

        // Force cache expiry but don't modify files
        if let Some(ref mut cache) = reader.cache {
            cache.timestamp = SystemTime::now() - Duration::from_secs(6 * 60);
        }

        // Second read - should reuse cached parse results (files unchanged)
        let metrics2 = reader.get_usage().expect("Should read from cache");
        assert_eq!(metrics2.total_input_tokens, 300, "Should have same values");

        // Cache should still contain both files
        assert_eq!(reader.cache.as_ref().unwrap().files.len(), 2);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 9: Filter files to today only
    #[test]
    fn test_reader_filter_today_only() {
        use std::time::Duration;

        let test_dir = create_test_dir("filter_today");

        // Create files
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);
        create_usage_file(&test_dir, "file2", 200, 100, 0.50);

        // Manually set file1's modification time to yesterday
        let file1_path = test_dir.join("file1.json");
        let yesterday = SystemTime::now() - Duration::from_secs(25 * 60 * 60); // 25 hours ago
        filetime::set_file_mtime(&file1_path, filetime::FileTime::from_system_time(yesterday))
            .expect("Failed to set file time");

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // Get today's usage only
        let metrics = reader.get_usage_today().expect("Should read today's data");

        // Should only include file2 (created today)
        assert_eq!(
            metrics.total_input_tokens, 200,
            "Should only count today's file"
        );
        assert_eq!(metrics.total_output_tokens, 100);
        assert_eq!(metrics.interaction_count, 1);
        assert!((metrics.total_cost - 0.50).abs() < 0.001);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 10: Filter today when no files from today exist
    #[test]
    fn test_reader_filter_today_no_data() {
        use std::time::Duration;

        let test_dir = create_test_dir("filter_today_no_data");

        // Create a file and set it to yesterday
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);

        let file1_path = test_dir.join("file1.json");
        let yesterday = SystemTime::now() - Duration::from_secs(25 * 60 * 60);
        filetime::set_file_mtime(&file1_path, filetime::FileTime::from_system_time(yesterday))
            .expect("Failed to set file time");

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // Try to get today's usage
        let result = reader.get_usage_today();

        // Should return error since no files from today
        assert!(result.is_err(), "Should error when no today data");
        assert!(matches!(result.unwrap_err(), ReaderError::NoDataFound));

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 11: All-time usage includes all files regardless of date
    #[test]
    fn test_reader_all_time_includes_all_files() {
        use std::time::Duration;

        let test_dir = create_test_dir("all_time");

        // Create files
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);
        create_usage_file(&test_dir, "file2", 200, 100, 0.50);
        create_usage_file(&test_dir, "file3", 150, 75, 0.30);

        // Set different modification times
        let yesterday = SystemTime::now() - Duration::from_secs(25 * 60 * 60);
        let two_days_ago = SystemTime::now() - Duration::from_secs(49 * 60 * 60);

        filetime::set_file_mtime(
            test_dir.join("file1.json"),
            filetime::FileTime::from_system_time(yesterday),
        )
        .expect("Failed to set time");

        filetime::set_file_mtime(
            test_dir.join("file2.json"),
            filetime::FileTime::from_system_time(two_days_ago),
        )
        .expect("Failed to set time");

        // file3 stays with today's date

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // Get all-time usage
        let metrics = reader.get_usage().expect("Should read all data");

        // Should include all files
        assert_eq!(metrics.total_input_tokens, 450);
        assert_eq!(metrics.total_output_tokens, 225);
        assert_eq!(metrics.interaction_count, 3);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 12: Cache should be preserved when switching between Today and AllTime modes
    #[test]
    fn test_reader_cache_preserved_across_mode_switches() {
        use std::time::Duration;

        let test_dir = create_test_dir("mode_switch_cache");

        // Create multiple files - some from today, some from yesterday
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);
        create_usage_file(&test_dir, "file2", 200, 100, 0.50);
        create_usage_file(&test_dir, "file3", 150, 75, 0.30);

        // Set file1 to yesterday
        let yesterday = SystemTime::now() - Duration::from_secs(25 * 60 * 60);
        filetime::set_file_mtime(
            test_dir.join("file1.json"),
            filetime::FileTime::from_system_time(yesterday),
        )
        .expect("Failed to set time");

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // Step 1: Get all-time usage (should parse all 3 files)
        let alltime_metrics = reader.get_usage().expect("Should read all-time data");
        assert_eq!(alltime_metrics.total_input_tokens, 450);
        assert_eq!(alltime_metrics.interaction_count, 3);

        // Verify cache has 3 files
        assert!(reader.cache.is_some());
        let cache_size_after_alltime = reader.cache.as_ref().unwrap().files.len();
        assert_eq!(cache_size_after_alltime, 3, "Cache should have all 3 files");

        // Step 2: Get today's usage (should reuse cached files for file2 and file3)
        let today_metrics = reader.get_usage_today().expect("Should read today's data");
        assert_eq!(
            today_metrics.total_input_tokens, 350,
            "Should only count today's files"
        );
        assert_eq!(today_metrics.interaction_count, 2);

        // Check cache after get_usage_today()
        eprintln!(
            "Cache after get_usage_today: exists={}, files={}",
            reader.cache.is_some(),
            reader.cache.as_ref().map_or(0, |c| c.files.len())
        );

        // The problem: parse_and_aggregate() doesn't update the cache
        // So the old cache (with all 3 files) is still there

        // Step 3: Get all-time usage again - should reuse cache, not re-parse
        // Force cache expiry for metrics, but file cache should remain
        if let Some(ref mut cache) = reader.cache {
            cache.timestamp = SystemTime::now() - Duration::from_secs(6 * 60);
        }

        let alltime_metrics2 = reader.get_usage().expect("Should read all-time data again");
        assert_eq!(alltime_metrics2.total_input_tokens, 450);

        // The cache should STILL have all 3 files parsed
        // This would prevent re-parsing when switching back to AllTime mode
        assert!(reader.cache.is_some(), "Cache should exist");
        let cache_size_after_switch = reader.cache.as_ref().unwrap().files.len();
        assert_eq!(
            cache_size_after_switch, 3,
            "File-level cache should be preserved across mode switches"
        );

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 13: Rapid mode switching should not cause full re-parsing
    #[test]
    fn test_reader_rapid_mode_switching_uses_cache() {
        use std::time::Duration;

        let test_dir = create_test_dir("rapid_switch");

        // Create many files to simulate real-world scenario
        for i in 0..100 {
            create_usage_file(&test_dir, &format!("file{i}"), 100, 50, 0.25);
        }

        // Set some files to yesterday
        let yesterday = SystemTime::now() - Duration::from_secs(25 * 60 * 60);
        for i in 0..50 {
            filetime::set_file_mtime(
                test_dir.join(format!("file{i}.json")),
                filetime::FileTime::from_system_time(yesterday),
            )
            .expect("Failed to set time");
        }

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // First: Get all-time usage (parses all 100 files)
        let start = std::time::Instant::now();
        let alltime_metrics = reader.get_usage().expect("Should read all-time data");
        let first_duration = start.elapsed();
        eprintln!("First all-time read: {first_duration:?}");
        assert_eq!(alltime_metrics.interaction_count, 100);
        assert_eq!(reader.cache.as_ref().unwrap().files.len(), 100);

        // Second: Get today's usage (should reuse cache for today's 50 files)
        let start = std::time::Instant::now();
        let today_metrics = reader.get_usage_today().expect("Should read today's data");
        let today_duration = start.elapsed();
        eprintln!("Today read: {today_duration:?}");
        assert_eq!(today_metrics.interaction_count, 50);

        // Third: Get all-time again (should be FAST - cache not expired yet)
        let start = std::time::Instant::now();
        let alltime_metrics2 = reader.get_usage().expect("Should read all-time data");
        let second_duration = start.elapsed();
        eprintln!("Second all-time read (cached): {second_duration:?}");
        assert_eq!(alltime_metrics2.interaction_count, 100);

        // The second all-time read should be much faster (using cache)
        assert!(
            second_duration < first_duration / 10,
            "Second read should be at least 10x faster: first={first_duration:?}, second={second_duration:?}"
        );

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 14: Filter files to this month only
    #[test]
    fn test_reader_filter_month_only() {
        use std::time::Duration;

        let test_dir = create_test_dir("filter_month");

        // Create files
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);
        create_usage_file(&test_dir, "file2", 200, 100, 0.50);
        create_usage_file(&test_dir, "file3", 150, 75, 0.30);

        // Set file1 to 31 days ago (outside this month)
        let file1_path = test_dir.join("file1.json");
        let last_month = SystemTime::now() - Duration::from_secs(31 * 24 * 60 * 60);
        filetime::set_file_mtime(
            &file1_path,
            filetime::FileTime::from_system_time(last_month),
        )
        .expect("Failed to set file time");

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // Get this month's usage only
        let metrics = reader
            .get_usage_month()
            .expect("Should read this month's data");

        // Should only include file2 and file3 (within this month)
        assert_eq!(
            metrics.total_input_tokens, 350,
            "Should only count this month's files"
        );
        assert_eq!(metrics.total_output_tokens, 175);
        assert_eq!(metrics.interaction_count, 2);
        assert!((metrics.total_cost - 0.80).abs() < 0.001);

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 15: Filter month when no files from this month exist
    #[test]
    fn test_reader_filter_month_no_data() {
        use std::time::Duration;

        let test_dir = create_test_dir("filter_month_no_data");

        // Create a file and set it to 31 days ago
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);

        let file1_path = test_dir.join("file1.json");
        let last_month = SystemTime::now() - Duration::from_secs(31 * 24 * 60 * 60);
        filetime::set_file_mtime(
            &file1_path,
            filetime::FileTime::from_system_time(last_month),
        )
        .expect("Failed to set file time");

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // Try to get this month's usage
        let result = reader.get_usage_month();

        // Should return error since no files from this month
        assert!(result.is_err(), "Should error when no month data");
        assert!(matches!(result.unwrap_err(), ReaderError::NoDataFound));

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 16: Verify calendar month start calculation
    #[test]
    fn test_month_start_calculation() {
        use chrono::{Datelike, Local, Timelike};
        use std::time::UNIX_EPOCH;

        let month_start = OpenCodeUsageReader::get_month_start();

        // Convert SystemTime to chrono DateTime in local timezone
        let month_start_duration = month_start
            .duration_since(UNIX_EPOCH)
            .expect("Should get duration");
        let month_start_secs = month_start_duration.as_secs() as i64;
        let month_start_dt = chrono::Local
            .timestamp_opt(month_start_secs, 0)
            .single()
            .expect("Should convert to DateTime");

        let now = Local::now();

        // Verify it's the first day of the current month
        assert_eq!(month_start_dt.day(), 1, "Should be first day of month");
        assert_eq!(
            month_start_dt.month(),
            now.month(),
            "Should be current month"
        );
        assert_eq!(month_start_dt.year(), now.year(), "Should be current year");

        // Verify it's at midnight (00:00:00)
        assert_eq!(month_start_dt.hour(), 0, "Should be at midnight");
        assert_eq!(month_start_dt.minute(), 0, "Should be at midnight");
        assert_eq!(month_start_dt.second(), 0, "Should be at midnight");
    }

    // Test 17: Verify month filtering uses calendar month
    #[test]
    fn test_reader_filter_calendar_month() {
        use chrono::{Datelike, Local, TimeZone};

        let test_dir = create_test_dir("filter_calendar_month");

        let now = Local::now();

        // Create files with different timestamps
        create_usage_file(&test_dir, "current_month", 100, 50, 0.25);
        create_usage_file(&test_dir, "last_month", 200, 100, 0.50);
        create_usage_file(&test_dir, "two_months_ago", 150, 75, 0.30);

        // Set current_month to a recent date within this month
        let current_file = test_dir.join("current_month.json");
        let this_month_time = Local
            .with_ymd_and_hms(now.year(), now.month(), 5, 12, 0, 0)
            .single()
            .expect("Should create valid date")
            .timestamp() as u64;
        filetime::set_file_mtime(
            &current_file,
            filetime::FileTime::from_unix_time(this_month_time as i64, 0),
        )
        .expect("Failed to set file time");

        // Set last_month to last month (even if only 25 days ago)
        let last_file = test_dir.join("last_month.json");
        let last_month = if now.month() == 1 {
            Local.with_ymd_and_hms(now.year() - 1, 12, 15, 12, 0, 0)
        } else {
            Local.with_ymd_and_hms(now.year(), now.month() - 1, 15, 12, 0, 0)
        }
        .single()
        .expect("Should create valid date");
        filetime::set_file_mtime(
            &last_file,
            filetime::FileTime::from_unix_time(last_month.timestamp(), 0),
        )
        .expect("Failed to set file time");

        // Set two_months_ago appropriately
        let two_months_file = test_dir.join("two_months_ago.json");
        let two_months_ago = if now.month() <= 2 {
            Local.with_ymd_and_hms(
                now.year() - 1,
                (12 + now.month() as i32 - 2) as u32,
                1,
                12,
                0,
                0,
            )
        } else {
            Local.with_ymd_and_hms(now.year(), now.month() - 2, 1, 12, 0, 0)
        }
        .single()
        .expect("Should create valid date");
        filetime::set_file_mtime(
            &two_months_file,
            filetime::FileTime::from_unix_time(two_months_ago.timestamp(), 0),
        )
        .expect("Failed to set file time");

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);

        // Get this month's usage
        let metrics = reader
            .get_usage_month()
            .expect("Should read this month's data");

        // Should only include current_month file
        assert_eq!(
            metrics.total_input_tokens, 100,
            "Should only count current calendar month files"
        );
        assert_eq!(metrics.total_output_tokens, 50);
        assert_eq!(metrics.interaction_count, 1);
        assert!((metrics.total_cost - 0.25).abs() < 0.001);

        fs::remove_dir_all(test_dir).ok();
    }
}
