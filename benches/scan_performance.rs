use cosmic_applet_opencode_usage::core::opencode::OpenCodeUsageReader;
use std::time::Instant;

fn main() {
    println!("Testing real-world performance with actual OpenCode storage...\n");
    
    match OpenCodeUsageReader::new() {
        Ok(mut reader) => {
            // Test 1: First all-time read (cold)
            let start = Instant::now();
            match reader.get_usage() {
                Ok(metrics) => {
                    let duration = start.elapsed();
                    println!("✓ First all-time read: {:?}", duration);
                    println!("  Files parsed: {}", metrics.interaction_count);
                    println!("  Total cost: ${:.2}", metrics.total_cost);
                }
                Err(e) => println!("✗ Error reading all-time: {}", e),
            }
            
            // Test 2: Today's usage
            let start = Instant::now();
            match reader.get_usage_today() {
                Ok(metrics) => {
                    let duration = start.elapsed();
                    println!("\n✓ Today's usage read: {:?}", duration);
                    println!("  Files parsed: {}", metrics.interaction_count);
                    println!("  Total cost: ${:.2}", metrics.total_cost);
                }
                Err(e) => println!("\n✗ Error reading today: {}", e),
            }
            
            // Test 3: All-time again (should be cached)
            let start = Instant::now();
            match reader.get_usage() {
                Ok(metrics) => {
                    let duration = start.elapsed();
                    println!("\n✓ Second all-time read (cached): {:?}", duration);
                    println!("  Files parsed: {}", metrics.interaction_count);
                }
                Err(e) => println!("\n✗ Error reading all-time: {}", e),
            }
        }
        Err(e) => {
            println!("Failed to create reader: {}", e);
        }
    }
}
