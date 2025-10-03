// SPDX-License-Identifier: GPL-3.0-only

//! Example to add varied test data spanning multiple days for chart visualization.

use chrono::{Duration, Utc};
use cosmic_applet_opencode_usage::core::database::DatabaseManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_manager = DatabaseManager::new()?;

    println!("Adding varied test data for last 30 days...\n");

    let now = Utc::now();

    // Create data for each of the last 30 days with varying amounts
    for days_ago in (0..30).rev() {
        let date = now - Duration::days(days_ago);
        let date_str = date.format("%Y-%m-%d").to_string();
        let created_at = date.to_rfc3339();

        // Vary the token counts to create an interesting trend
        // Simulate increasing usage over time
        let base_input = 5000 + (days_ago * 200);
        let base_output = 2000 + (days_ago * 150);
        let base_reasoning = 1000 + (days_ago * 50);

        // Add some randomness (using day as seed for consistency)
        let variation = (days_ago % 7) * 100;

        let input_tokens = base_input + variation;
        let output_tokens = base_output + variation;
        let reasoning_tokens = base_reasoning + variation / 2;
        let cache_write = input_tokens / 10;
        let cache_read = input_tokens / 20;
        let interactions = 5 + (days_ago % 5);

        // Calculate cost (rough estimate)
        #[allow(clippy::cast_precision_loss)] // Test data generation
        let cost = (input_tokens as f64 * 0.000_003)
            + (output_tokens as f64 * 0.000_015)
            + (reasoning_tokens as f64 * 0.000_003);

        let conn = db_manager.get_connection();
        conn.execute(
            "INSERT INTO usage_snapshots 
             (date, input_tokens, output_tokens, reasoning_tokens, 
              cache_write_tokens, cache_read_tokens, total_cost, 
              interaction_count, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                date_str,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                cache_write,
                cache_read,
                cost,
                interactions,
                created_at
            ],
        )?;

        println!(
            "Added data for {days_ago} days ago: Input={input_tokens}, Output={output_tokens}, Reasoning={reasoning_tokens}"
        );
    }

    println!("\n✅ Successfully added 30 days of varied test data!");
    println!("Now run: cargo run --bin cosmic-applet-opencode-usage-viewer");

    Ok(())
}
