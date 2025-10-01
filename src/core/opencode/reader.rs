use crate::core::opencode::{
    StorageScanner, ScannerError, UsageParser, 
    UsageAggregator, UsageMetrics, FileMetadata, UsagePart
};
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

/// Main orchestrator for reading OpenCode usage data
pub struct OpenCodeUsageReader {
    scanner: StorageScanner,
    cache: Option<CachedData>,
}

impl OpenCodeUsageReader {
    /// Create a new reader with default OpenCode storage path
    pub fn new() -> Result<Self, ReaderError> {
        let scanner = StorageScanner::new()?;
        Ok(Self {
            scanner,
            cache: None,
        })
    }
    
    /// Create a reader with a custom path (useful for testing)
    pub fn new_with_path(path: &str) -> Result<Self, ReaderError> {
        let scanner = StorageScanner::with_path(std::path::PathBuf::from(path))?;
        Ok(Self {
            scanner,
            cache: None,
        })
    }
    
    /// Create a reader with a custom scanner (useful for testing)
    pub fn with_scanner(scanner: StorageScanner) -> Self {
        Self {
            scanner,
            cache: None,
        }
    }
    
    /// Get usage metrics, using cache if available and not expired
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
    pub fn get_usage_today(&mut self) -> Result<UsageMetrics, ReaderError> {
        // Scan all files with metadata
        let all_files = self.scanner.scan_with_metadata()?;
        
        // Filter to only files modified today
        let today_files = Self::filter_today_files(&all_files);
        
        if today_files.is_empty() {
            return Err(ReaderError::NoDataFound);
        }
        
        // Parse and aggregate filtered files
        self.parse_and_aggregate(&today_files)
    }
    
    /// Helper function to check if a SystemTime is from today
    fn is_today(time: SystemTime) -> bool {
        let now = SystemTime::now();
        
        // Get duration since UNIX_EPOCH for both times
        let time_since_epoch = time.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        let now_since_epoch = now.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        
        // Convert to seconds
        let time_secs = time_since_epoch.as_secs();
        let now_secs = now_since_epoch.as_secs();
        
        // Calculate start of today (midnight) in seconds since epoch
        // 86400 seconds = 24 hours
        let today_start_secs = (now_secs / 86400) * 86400;
        
        // Check if time is after start of today
        time_secs >= today_start_secs && time_secs <= now_secs
    }
    
    /// Filter files to only those modified today
    fn filter_today_files(files: &[FileMetadata]) -> Vec<FileMetadata> {
        files.iter()
            .filter(|file| Self::is_today(file.modified))
            .cloned()
            .collect()
    }
    
    /// Parse and aggregate usage files (shared logic for get_usage and get_usage_today)
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
    fn incremental_parse(&self, files: &[FileMetadata]) -> Result<(Vec<UsagePart>, HashMap<PathBuf, CachedFile>), ReaderError> {
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
                match UsageParser::parse_file(&file_meta.path) {
                    Ok(Some(part)) => {
                        parts.push(part.clone());
                        new_cache.insert(
                            file_meta.path.clone(),
                            CachedFile {
                                part,
                                modified: file_meta.modified,
                            },
                        );
                    }
                    Ok(None) => {
                        // File parsed but no tokens - skip silently
                    }
                    Err(_) => {
                        // Invalid JSON - skip silently
                    }
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
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::fs;
    use std::io::Write;

    /// Helper to create a temporary test directory
    fn create_test_dir(name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir()
            .join(format!("opencode_reader_test_{}", name));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("Failed to create test directory");
        temp_dir
    }

    /// Helper to create a usage part JSON file
    fn create_usage_file(dir: &PathBuf, name: &str, input: u64, output: u64, cost: f64) {
        let content = format!(
            r#"{{
                "id": "prt_{}",
                "messageID": "msg_test",
                "sessionID": "ses_test",
                "type": "step-finish",
                "tokens": {{
                    "input": {},
                    "output": {},
                    "reasoning": 0,
                    "cache": {{
                        "write": 0,
                        "read": 0
                    }}
                }},
                "cost": {}
            }}"#,
            name, input, output, cost
        );
        
        let file_path = dir.join(format!("{}.json", name));
        let mut file = fs::File::create(file_path).expect("Failed to create test file");
        file.write_all(content.as_bytes()).expect("Failed to write test file");
    }

    // Test 1: Read and aggregate sample data
    #[test]
    fn test_reader_with_sample_data() {
        let test_dir = create_test_dir("sample_data");
        
        // Create sample usage files
        create_usage_file(&test_dir, "file1", 100, 50, 0.25);
        create_usage_file(&test_dir, "file2", 200, 100, 0.50);
        create_usage_file(&test_dir, "file3", 150, 75, 0.30);
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
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
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
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
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);
        
        // First call - should scan
        let metrics1 = reader.get_usage().expect("Should read usage data");
        let timestamp1 = metrics1.timestamp;
        
        // Second call immediately - should use cache
        let metrics2 = reader.get_usage().expect("Should read usage data from cache");
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
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
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
        assert_eq!(metrics.total_input_tokens, 100, "Should not use expired cache");
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
        file.write_all(br#"{
            "id": "prt_test",
            "messageID": "msg_test",
            "sessionID": "ses_test",
            "type": "step-start",
            "cost": 0
        }"#).expect("Should write");
        drop(file);
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);
        
        let metrics = reader.get_usage().expect("Should read valid files despite invalid ones");
        
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
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
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
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
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
        assert_eq!(metrics2.total_input_tokens, 500, "Should have updated values from modified file");
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
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
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
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);
        
        // Get today's usage only
        let metrics = reader.get_usage_today().expect("Should read today's data");
        
        // Should only include file2 (created today)
        assert_eq!(metrics.total_input_tokens, 200, "Should only count today's file");
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
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
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
            &test_dir.join("file1.json"),
            filetime::FileTime::from_system_time(yesterday)
        ).expect("Failed to set time");
        
        filetime::set_file_mtime(
            &test_dir.join("file2.json"),
            filetime::FileTime::from_system_time(two_days_ago)
        ).expect("Failed to set time");
        
        // file3 stays with today's date
        
        let scanner = StorageScanner::with_path(test_dir.clone())
            .expect("Should create scanner");
        let mut reader = OpenCodeUsageReader::with_scanner(scanner);
        
        // Get all-time usage
        let metrics = reader.get_usage().expect("Should read all data");
        
        // Should include all files
        assert_eq!(metrics.total_input_tokens, 450);
        assert_eq!(metrics.total_output_tokens, 225);
        assert_eq!(metrics.interaction_count, 3);
        
        fs::remove_dir_all(test_dir).ok();
    }
}
