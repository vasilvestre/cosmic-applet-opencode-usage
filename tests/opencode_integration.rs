use cosmic_applet_opencode_usage::core::opencode::OpenCodeUsageReader;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a realistic OpenCode usage JSON file
fn create_usage_file(dir: &PathBuf, filename: &str, input_tokens: u64, output_tokens: u64, cost: f64) {
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
        filename.replace(".json", ""),
        input_tokens,
        output_tokens,
        cost
    );
    fs::write(dir.join(filename), content).expect("Failed to write test file");
}

/// Helper function to create a usage file with cache data
fn create_usage_file_with_cache(
    dir: &PathBuf,
    filename: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
    cost: f64,
) {
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
      "write": {},
      "read": {}
    }}
  }},
  "cost": {}
}}"#,
        filename.replace(".json", ""),
        input_tokens,
        output_tokens,
        cache_creation_tokens,
        cache_read_tokens,
        cost
    );
    fs::write(dir.join(filename), content).expect("Failed to write test file");
}

/// Helper function to create a usage file without tokens (should be skipped)
fn create_usage_file_without_tokens(dir: &PathBuf, filename: &str) {
    let content = r#"{
  "id": "prt_test_no_tokens",
  "messageID": "msg_test",
  "sessionID": "ses_test",
  "type": "step-start",
  "cost": 0.0
}"#;
    fs::write(dir.join(filename), content).expect("Failed to write test file");
}

#[test]
fn test_integration_realistic_opencode_structure() {
    // RED: Write the test first
    // Create a temporary directory structure mimicking OpenCode storage
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().to_path_buf();

    // Simulate OpenCode's nested structure: storage/conversations/{uuid}/usage-{uuid}.json
    let conv1 = storage_path.join("conversations/conv-001");
    let conv2 = storage_path.join("conversations/conv-002");
    let conv3 = storage_path.join("conversations/conv-003");

    fs::create_dir_all(&conv1).expect("Failed to create conv1");
    fs::create_dir_all(&conv2).expect("Failed to create conv2");
    fs::create_dir_all(&conv3).expect("Failed to create conv3");

    // Populate with realistic usage files
    // Conversation 1: 3 interactions
    create_usage_file(&conv1, "usage-001-01.json", 1000, 500, 0.015);
    create_usage_file(&conv1, "usage-001-02.json", 1200, 800, 0.020);
    create_usage_file(&conv1, "usage-001-03.json", 900, 600, 0.015);

    // Conversation 2: 2 interactions with cache
    create_usage_file_with_cache(&conv2, "usage-002-01.json", 800, 400, 2000, 1500, 0.025);
    create_usage_file_with_cache(&conv2, "usage-002-02.json", 700, 350, 1800, 1200, 0.022);

    // Conversation 3: 1 interaction + 1 file without tokens (should be skipped)
    create_usage_file(&conv3, "usage-003-01.json", 1500, 1000, 0.030);
    create_usage_file_without_tokens(&conv3, "usage-003-02.json");

    // Also add some non-JSON files that should be ignored
    fs::write(conv1.join("metadata.txt"), "some metadata").expect("Failed to write metadata");
    fs::write(conv2.join("README.md"), "# Readme").expect("Failed to write readme");

    // GREEN: Now implement the reader to make this test pass
    let mut reader = OpenCodeUsageReader::new_with_path(storage_path.to_str().unwrap())
        .expect("Failed to create reader");

    let metrics = reader.get_usage().expect("Failed to get usage");

    // Verify aggregated metrics
    // Expected calculations:
    // Input tokens: 1000 + 1200 + 900 + 800 + 700 + 1500 = 6100
    // Output tokens: 500 + 800 + 600 + 400 + 350 + 1000 = 3650
    // Cache creation: 2000 + 1800 = 3800
    // Cache read: 1500 + 1200 = 2700
    // Total cost: 0.015 + 0.020 + 0.015 + 0.025 + 0.022 + 0.030 = 0.127
    // Interactions: 6 (files with token data)

    assert_eq!(metrics.total_input_tokens, 6100, "Input tokens mismatch");
    assert_eq!(metrics.total_output_tokens, 3650, "Output tokens mismatch");
    assert_eq!(
        metrics.total_cache_write_tokens, 3800,
        "Cache creation tokens mismatch"
    );
    assert_eq!(metrics.total_cache_read_tokens, 2700, "Cache read tokens mismatch");
    assert_eq!(metrics.interaction_count, 6, "Interactions count mismatch");

    // Cost comparison with floating point tolerance
    let expected_cost = 0.127;
    let diff = (metrics.total_cost - expected_cost).abs();
    assert!(
        diff < 0.0001,
        "Cost mismatch: expected {}, got {}",
        expected_cost,
        metrics.total_cost
    );

    // Verify timestamp is set
    assert!(
        metrics.timestamp.elapsed().is_ok(),
        "Timestamp should be set to a valid system time"
    );
}

#[test]
fn test_integration_nested_directory_structure() {
    // Test with deeply nested directories (OpenCode can have many levels)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().to_path_buf();

    // Create 5 levels deep nested structure
    let deep_path = storage_path
        .join("level1")
        .join("level2")
        .join("level3")
        .join("level4")
        .join("level5");

    fs::create_dir_all(&deep_path).expect("Failed to create nested structure");

    // Add usage files at different levels
    create_usage_file(&storage_path.join("level1"), "usage-l1.json", 100, 50, 0.003);
    create_usage_file(
        &storage_path.join("level1/level2"),
        "usage-l2.json",
        200,
        100,
        0.006,
    );
    create_usage_file(&deep_path, "usage-l5.json", 300, 150, 0.009);

    let mut reader = OpenCodeUsageReader::new_with_path(storage_path.to_str().unwrap())
        .expect("Failed to create reader");

    let metrics = reader.get_usage().expect("Failed to get usage");

    // Should find all 3 files regardless of nesting level
    assert_eq!(metrics.total_input_tokens, 600);
    assert_eq!(metrics.total_output_tokens, 300);
    assert_eq!(metrics.interaction_count, 3);

    let expected_cost = 0.018;
    let diff = (metrics.total_cost - expected_cost).abs();
    assert!(diff < 0.0001, "Cost mismatch");
}

#[test]
fn test_integration_with_invalid_json_files() {
    // Test resilience to corrupted/invalid JSON files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().to_path_buf();

    fs::create_dir_all(&storage_path).expect("Failed to create temp dir");

    // Valid files
    create_usage_file(&storage_path, "valid-01.json", 1000, 500, 0.015);
    create_usage_file(&storage_path, "valid-02.json", 1500, 750, 0.023);

    // Invalid JSON files
    fs::write(storage_path.join("invalid-01.json"), "{broken json")
        .expect("Failed to write invalid file");
    fs::write(storage_path.join("invalid-02.json"), "not json at all")
        .expect("Failed to write invalid file");
    fs::write(storage_path.join("empty.json"), "").expect("Failed to write empty file");

    let mut reader = OpenCodeUsageReader::new_with_path(storage_path.to_str().unwrap())
        .expect("Failed to create reader");

    // Should succeed despite invalid files
    let metrics = reader.get_usage().expect("Failed to get usage despite invalid files");

    // Should only count valid files
    assert_eq!(metrics.total_input_tokens, 2500);
    assert_eq!(metrics.total_output_tokens, 1250);
    assert_eq!(metrics.interaction_count, 2);
}

#[test]
fn test_integration_empty_storage() {
    // Test with empty storage directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().to_path_buf();

    fs::create_dir_all(&storage_path).expect("Failed to create temp dir");

    let mut reader = OpenCodeUsageReader::new_with_path(storage_path.to_str().unwrap())
        .expect("Failed to create reader");

    let result = reader.get_usage();

    // Should return error when no data found
    assert!(result.is_err(), "Empty storage should return an error");
}

#[test]
fn test_integration_caching_behavior() {
    // Test that caching works correctly across multiple calls
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().to_path_buf();

    fs::create_dir_all(&storage_path).expect("Failed to create temp dir");

    create_usage_file(&storage_path, "usage-01.json", 1000, 500, 0.015);

    let mut reader = OpenCodeUsageReader::new_with_path(storage_path.to_str().unwrap())
        .expect("Failed to create reader");

    // First call - should scan
    let metrics1 = reader.get_usage().expect("First call failed");
    assert_eq!(metrics1.total_input_tokens, 1000);

    // Add another file
    create_usage_file(&storage_path, "usage-02.json", 2000, 1000, 0.030);

    // Second call immediately - should use cache (won't see new file)
    let metrics2 = reader.get_usage().expect("Second call failed");
    assert_eq!(
        metrics2.total_input_tokens, 1000,
        "Should still see cached value"
    );

    // Force cache expiry by waiting or manipulating time (for real test, we'd need to wait 5 min)
    // For now, just verify that the cache mechanism exists
    assert_eq!(metrics1.total_input_tokens, metrics2.total_input_tokens);
}

#[test]
fn test_integration_large_dataset() {
    // Test with larger dataset (simulate 100 files)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().to_path_buf();

    fs::create_dir_all(&storage_path).expect("Failed to create temp dir");

    // Create 100 usage files with varying token counts
    let mut expected_input = 0u64;
    let mut expected_output = 0u64;
    let mut expected_cost = 0.0f64;

    for i in 0..100 {
        let input = 1000 + (i * 10) as u64;
        let output = 500 + (i * 5) as u64;
        let cost = 0.015 + (i as f64 * 0.0001);

        expected_input += input;
        expected_output += output;
        expected_cost += cost;

        create_usage_file(
            &storage_path,
            &format!("usage-{:03}.json", i),
            input,
            output,
            cost,
        );
    }

    let mut reader = OpenCodeUsageReader::new_with_path(storage_path.to_str().unwrap())
        .expect("Failed to create reader");

    let metrics = reader.get_usage().expect("Failed to get usage");

    assert_eq!(metrics.total_input_tokens, expected_input);
    assert_eq!(metrics.total_output_tokens, expected_output);
    assert_eq!(metrics.interaction_count, 100);

    // Cost comparison with tolerance
    let diff = (metrics.total_cost - expected_cost).abs();
    assert!(
        diff < 0.001,
        "Cost mismatch: expected {}, got {}",
        expected_cost,
        metrics.total_cost
    );
}
