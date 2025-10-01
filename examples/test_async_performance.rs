use cosmic_applet_opencode_usage::core::opencode::OpenCodeUsageReader;
use std::time::Instant;

fn main() {
    println!("Testing async performance with actual OpenCode storage...\n");
    
    match OpenCodeUsageReader::new() {
        Ok(mut reader) => {
            println!("✓ Created reader");
            
            // Test 1: First all-time read (cold cache)
            println!("\n--- Test 1: First all-time read (cold cache) ---");
            let start = Instant::now();
            match reader.get_usage() {
                Ok(metrics) => {
                    let duration = start.elapsed();
                    println!("✓ Duration: {:?}", duration);
                    println!("  Files: {}", metrics.interaction_count);
                    println!("  Cost: ${:.2}", metrics.total_cost);
                }
                Err(e) => println!("✗ Error: {}", e),
            }
            
            // Test 2: Today's usage
            println!("\n--- Test 2: Today's usage ---");
            let start = Instant::now();
            match reader.get_usage_today() {
                Ok(metrics) => {
                    let duration = start.elapsed();
                    println!("✓ Duration: {:?}", duration);
                    println!("  Files: {}", metrics.interaction_count);
                    println!("  Cost: ${:.2}", metrics.total_cost);
                }
                Err(e) => println!("✗ Error: {}", e),
            }
            
            // Test 3: Switch back to all-time (should use cache)
            println!("\n--- Test 3: All-time again (cached) ---");
            let start = Instant::now();
            match reader.get_usage() {
                Ok(metrics) => {
                    let duration = start.elapsed();
                    println!("✓ Duration: {:?}", duration);
                    println!("  Files: {}", metrics.interaction_count);
                    println!("  Cost: ${:.2}", metrics.total_cost);
                }
                Err(e) => println!("✗ Error: {}", e),
            }
            
            // Test 4: Multiple rapid switches
            println!("\n--- Test 4: Rapid switching (10 iterations) ---");
            let start = Instant::now();
            for i in 0..10 {
                if i % 2 == 0 {
                    let _ = reader.get_usage();
                } else {
                    let _ = reader.get_usage_today();
                }
            }
            let duration = start.elapsed();
            println!("✓ Total duration for 10 switches: {:?}", duration);
            println!("  Average per switch: {:?}", duration / 10);
            
            println!("\n--- Summary ---");
            println!("The async implementation should prevent UI freezing by:");
            println!("1. Moving directory scanning to a background thread");
            println!("2. Allowing the UI to remain responsive during data fetch");
            println!("3. Showing loading state while fetch is in progress");
        }
        Err(e) => {
            println!("✗ Failed to create reader: {}", e);
        }
    }
}
