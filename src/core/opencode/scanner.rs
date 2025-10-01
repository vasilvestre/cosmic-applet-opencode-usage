use rayon::prelude::*;
use std::path::PathBuf;
use std::time::SystemTime;
use thiserror::Error;
use walkdir::WalkDir;

/// Error types for scanning operations
#[derive(Debug, Error)]
pub enum ScannerError {
    #[error("Storage directory not found: {0}")]
    DirectoryNotFound(PathBuf),

    #[error("Failed to access storage directory: {0}")]
    AccessError(String),

    #[error("Walk directory error: {0}")]
    WalkError(#[from] walkdir::Error),

    #[error("Failed to get file metadata: {0}")]
    MetadataError(String),
}

/// File metadata for caching decisions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub modified: SystemTime,
}

/// Scans `OpenCode` storage directory for usage part files
#[derive(Debug)]
pub struct StorageScanner {
    storage_path: PathBuf,
}

impl StorageScanner {
    /// Create a new scanner with the default `OpenCode` storage path
    ///
    /// # Errors
    /// Returns an error if the HOME environment variable is not set or the storage path doesn't exist.
    pub fn new() -> Result<Self, ScannerError> {
        let home = std::env::var("HOME")
            .map_err(|e| ScannerError::AccessError(format!("Cannot get HOME: {e}")))?;

        let storage_path = PathBuf::from(home).join(".local/share/opencode/storage/part");

        Self::with_path(storage_path)
    }

    /// Create a scanner with a custom storage path (useful for testing)
    ///
    /// # Errors
    /// Returns an error if the storage path doesn't exist.
    pub fn with_path(storage_path: PathBuf) -> Result<Self, ScannerError> {
        if !storage_path.exists() {
            return Err(ScannerError::DirectoryNotFound(storage_path));
        }

        Ok(Self { storage_path })
    }

    /// Scan the storage directory and return paths to all JSON files
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read or accessed.
    pub fn scan(&self) -> Result<Vec<PathBuf>, ScannerError> {
        let json_files = WalkDir::new(&self.storage_path)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    Some(path.to_path_buf())
                } else {
                    None
                }
            })
            .collect();

        Ok(json_files)
    }

    /// Get the storage path
    #[must_use]
    pub fn storage_path(&self) -> &PathBuf {
        &self.storage_path
    }

    /// Scan the storage directory and return file metadata (path + modified time)
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read or accessed.
    pub fn scan_with_metadata(&self) -> Result<Vec<FileMetadata>, ScannerError> {
        // First, collect all directory entries (fast I/O operation)
        let entries: Vec<_> = WalkDir::new(&self.storage_path)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .collect();

        // Then, process entries in parallel using rayon
        let metadata: Vec<FileMetadata> = entries
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    // Get modification time
                    match entry.metadata() {
                        Ok(meta) => match meta.modified() {
                            Ok(modified) => Some(FileMetadata {
                                path: path.to_path_buf(),
                                modified,
                            }),
                            Err(_) => {
                                // Skip files we can't get modification time for
                                None
                            }
                        },
                        Err(_) => {
                            // Skip files we can't get metadata for
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(metadata)
    }

    /// Scan the storage directory and return only files modified after the cutoff time
    /// This is optimized to skip old files during the walk, reducing I/O overhead
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read or accessed.
    pub fn scan_modified_since(
        &self,
        cutoff: SystemTime,
    ) -> Result<Vec<FileMetadata>, ScannerError> {
        // First, collect all directory entries (fast I/O operation)
        let entries: Vec<_> = WalkDir::new(&self.storage_path)
            .follow_links(false)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .collect();

        // Then, process entries in parallel using rayon, filtering by modification time
        let metadata: Vec<FileMetadata> = entries
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                    // Get modification time
                    match entry.metadata() {
                        Ok(meta) => match meta.modified() {
                            Ok(modified) => {
                                // Only include files modified after cutoff
                                if modified >= cutoff {
                                    Some(FileMetadata {
                                        path: path.to_path_buf(),
                                        modified,
                                    })
                                } else {
                                    None
                                }
                            }
                            Err(_) => {
                                // Skip files we can't get modification time for
                                None
                            }
                        },
                        Err(_) => {
                            // Skip files we can't get metadata for
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    /// Helper to create a temporary test directory
    fn create_test_dir(name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("opencode_scanner_test_{name}"));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("Failed to create test directory");
        temp_dir
    }

    /// Helper to create a test file
    fn create_test_file(dir: &Path, name: &str, content: &str) {
        let file_path = dir.join(name);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut file = fs::File::create(file_path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test file");
    }

    // Test 1: Handle empty directory
    #[test]
    fn test_scanner_with_empty_directory() {
        let test_dir = create_test_dir("empty");

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let files = scanner.scan().expect("Should scan successfully");

        assert_eq!(files.len(), 0, "Empty directory should return no files");

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 2: Find all JSON files
    #[test]
    fn test_scanner_finds_json_files() {
        let test_dir = create_test_dir("find_json");

        create_test_file(&test_dir, "file1.json", r#"{"test": 1}"#);
        create_test_file(&test_dir, "file2.json", r#"{"test": 2}"#);
        create_test_file(&test_dir, "file3.json", r#"{"test": 3}"#);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let files = scanner.scan().expect("Should scan successfully");

        assert_eq!(files.len(), 3, "Should find all 3 JSON files");

        for file in &files {
            assert!(file.extension().is_some_and(|ext| ext == "json"));
        }

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 3: Filter out non-JSON files
    #[test]
    fn test_scanner_filters_non_json() {
        let test_dir = create_test_dir("filter_non_json");

        create_test_file(&test_dir, "data.json", r#"{"test": 1}"#);
        create_test_file(&test_dir, "readme.txt", "This is text");
        create_test_file(&test_dir, "image.png", "fake png data");
        create_test_file(&test_dir, "script.sh", "#!/bin/bash");
        create_test_file(&test_dir, "another.json", r#"{"test": 2}"#);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let files = scanner.scan().expect("Should scan successfully");

        assert_eq!(files.len(), 2, "Should find only 2 JSON files");

        for file in &files {
            assert!(file.extension().is_some_and(|ext| ext == "json"));
        }

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 4: Traverse nested directory structure
    #[test]
    fn test_scanner_nested_directories() {
        let test_dir = create_test_dir("nested");

        create_test_file(&test_dir, "root.json", r#"{"test": 1}"#);
        create_test_file(&test_dir, "subdir1/file1.json", r#"{"test": 2}"#);
        create_test_file(&test_dir, "subdir1/file2.json", r#"{"test": 3}"#);
        create_test_file(&test_dir, "subdir2/file3.json", r#"{"test": 4}"#);
        create_test_file(&test_dir, "subdir1/nested/file4.json", r#"{"test": 5}"#);
        create_test_file(
            &test_dir,
            "subdir2/nested/deep/file5.json",
            r#"{"test": 6}"#,
        );

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let files = scanner.scan().expect("Should scan successfully");

        assert_eq!(
            files.len(),
            6,
            "Should find all 6 JSON files in nested structure"
        );

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 5: Return error for nonexistent directory
    #[test]
    fn test_scanner_nonexistent_directory() {
        let nonexistent_path = PathBuf::from("/tmp/this_directory_should_not_exist_xyz123");

        let result = StorageScanner::with_path(nonexistent_path);

        assert!(
            result.is_err(),
            "Should return error for nonexistent directory"
        );
        assert!(matches!(
            result.unwrap_err(),
            ScannerError::DirectoryNotFound(_)
        ));
    }

    // Test 6: Scanner with nested structure and mixed file types
    #[test]
    fn test_scanner_complex_structure() {
        let test_dir = create_test_dir("complex");

        create_test_file(&test_dir, "data.json", r#"{"test": 1}"#);
        create_test_file(&test_dir, "readme.md", "# Readme");
        create_test_file(&test_dir, "level1/a.json", r#"{"test": 2}"#);
        create_test_file(&test_dir, "level1/b.txt", "text file");
        create_test_file(&test_dir, "level1/level2/c.json", r#"{"test": 3}"#);
        create_test_file(&test_dir, "level1/level2/d.log", "log file");

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let files = scanner.scan().expect("Should scan successfully");

        assert_eq!(files.len(), 3, "Should find exactly 3 JSON files");

        for file in &files {
            assert!(
                file.extension().is_some_and(|ext| ext == "json"),
                "Found non-JSON file: {file:?}"
            );
        }

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 7: scan_with_metadata returns file metadata
    #[test]
    fn test_scanner_with_metadata() {
        let test_dir = create_test_dir("metadata");

        create_test_file(&test_dir, "file1.json", r#"{"test": 1}"#);
        create_test_file(&test_dir, "file2.json", r#"{"test": 2}"#);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let metadata = scanner
            .scan_with_metadata()
            .expect("Should scan successfully");

        assert_eq!(metadata.len(), 2, "Should find 2 files with metadata");

        for file_meta in &metadata {
            assert!(file_meta.path.extension().is_some_and(|ext| ext == "json"));
            // Modification time should be recent (within last minute)
            let elapsed = file_meta
                .modified
                .elapsed()
                .expect("Should get elapsed time");
            assert!(
                elapsed.as_secs() < 60,
                "File should have recent modification time"
            );
        }

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 8: scan_with_metadata filters non-JSON files
    #[test]
    fn test_scanner_with_metadata_filters() {
        let test_dir = create_test_dir("metadata_filter");

        create_test_file(&test_dir, "data.json", r#"{"test": 1}"#);
        create_test_file(&test_dir, "readme.txt", "This is text");
        create_test_file(&test_dir, "another.json", r#"{"test": 2}"#);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");
        let metadata = scanner
            .scan_with_metadata()
            .expect("Should scan successfully");

        assert_eq!(metadata.len(), 2, "Should find only 2 JSON files");

        fs::remove_dir_all(test_dir).ok();
    }

    // Test 9: scan_modified_since filters files by modification time
    #[test]
    fn test_scanner_modified_since() {
        use std::time::Duration;

        let test_dir = create_test_dir("modified_since");

        // Create old files
        create_test_file(&test_dir, "old1.json", r#"{"test": 1}"#);
        create_test_file(&test_dir, "old2.json", r#"{"test": 2}"#);

        // Set modification time to 2 days ago
        let two_days_ago = SystemTime::now() - Duration::from_secs(48 * 60 * 60);
        filetime::set_file_mtime(
            test_dir.join("old1.json"),
            filetime::FileTime::from_system_time(two_days_ago),
        )
        .expect("Failed to set file time");
        filetime::set_file_mtime(
            test_dir.join("old2.json"),
            filetime::FileTime::from_system_time(two_days_ago),
        )
        .expect("Failed to set file time");

        // Sleep briefly to ensure different timestamp
        std::thread::sleep(Duration::from_millis(10));

        // Create recent files
        create_test_file(&test_dir, "recent1.json", r#"{"test": 3}"#);
        create_test_file(&test_dir, "recent2.json", r#"{"test": 4}"#);

        let scanner = StorageScanner::with_path(test_dir.clone()).expect("Should create scanner");

        // Scan only files modified in last 24 hours
        let cutoff = SystemTime::now() - Duration::from_secs(24 * 60 * 60);
        let metadata = scanner
            .scan_modified_since(cutoff)
            .expect("Should scan successfully");

        // Should only find the 2 recent files
        assert_eq!(metadata.len(), 2, "Should find only recent files");
        for file in &metadata {
            assert!(file.modified > cutoff, "All files should be after cutoff");
        }

        fs::remove_dir_all(test_dir).ok();
    }
}
