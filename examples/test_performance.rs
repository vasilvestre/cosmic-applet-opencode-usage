use cosmic_applet_opencode_usage::core::opencode::OpenCodeUsageReader;
use std::time::Instant;

fn main() {
    println!("=== OpenCode Usage Performance Test ===\n");

    // Create reader with real OpenCode storage
    let mut reader = match OpenCodeUsageReader::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error creating reader: {}", e);
            return;
        }
    };

    // Test 1: All-time usage (scans all 69K+ files)
    println!("Test 1: All-time usage (scanning all files)");
    let start = Instant::now();
    match reader.get_usage() {
        Ok(metrics) => {
            let duration = start.elapsed();
            println!("  ✓ Duration: {:?}", duration);
            println!("  ✓ Total interactions: {}", metrics.interaction_count);
            println!("  ✓ Total input tokens: {}", metrics.total_input_tokens);
        }
        Err(e) => {
            println!("  ✗ Error: {}", e);
        }
    }
    println!();

    // Test 2: Today's usage (with new optimized scan_modified_since)
    println!("Test 2: Today's usage (optimized filtering)");
    let start = Instant::now();
    match reader.get_usage_today() {
        Ok(metrics) => {
            let duration = start.elapsed();
            println!("  ✓ Duration: {:?}", duration);
            println!("  ✓ Today's interactions: {}", metrics.interaction_count);
            println!("  ✓ Today's input tokens: {}", metrics.total_input_tokens);
        }
        Err(e) => {
            println!("  ✗ Error: {}", e);
        }
    }
    println!();

    // Test 3: Rapid switching (should use cache)
    println!("Test 3: Switching back to all-time (should use cache)");
    let start = Instant::now();
    match reader.get_usage() {
        Ok(metrics) => {
            let duration = start.elapsed();
            println!("  ✓ Duration: {:?}", duration);
            println!("  ✓ Total interactions: {}", metrics.interaction_count);
        }
        Err(e) => {
            println!("  ✗ Error: {}", e);
        }
    }
    println!();

    // Test 4: Switch back to today (rapid switching)
    println!("Test 4: Switching back to today (rapid switch test)");
    let start = Instant::now();
    match reader.get_usage_today() {
        Ok(metrics) => {
            let duration = start.elapsed();
            println!("  ✓ Duration: {:?}", duration);
            println!("  ✓ Today's interactions: {}", metrics.interaction_count);
        }
        Err(e) => {
            println!("  ✗ Error: {}", e);
        }
    }

    println!("\n=== Performance Test Complete ===");
}
