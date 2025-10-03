// SPDX-License-Identifier: GPL-3.0-only

//! Manually trigger a data collection from OpenCode storage.

use cosmic_applet_opencode_usage::core::{
    database::DatabaseManager, opencode::OpenCodeUsageReader,
};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Manual Data Collection Tool\n");

    // Use the production database path
    let home = std::env::var("HOME")?;
    let db_path =
        std::path::PathBuf::from(&home).join(".local/share/cosmic-applet-opencode-usage/usage.db");

    println!("Database: {}", db_path.display());

    // Create/open database
    let db = Arc::new(DatabaseManager::new_with_path(&db_path)?);
    let repo = cosmic_applet_opencode_usage::core::database::repository::UsageRepository::new(
        Arc::clone(&db),
    );

    // Create reader with default OpenCode storage path
    let storage_path = std::path::PathBuf::from(&home).join(".local/share/opencode/storage/part");
    println!("Storage:  {}\n", storage_path.display());

    if !storage_path.exists() {
        eprintln!("❌ OpenCode storage directory not found!");
        eprintln!("Expected: {}", storage_path.display());
        return Ok(());
    }

    let mut reader = OpenCodeUsageReader::new()?;

    // Read all-time usage
    println!("📊 Reading all-time usage from OpenCode...");
    let metrics = reader.get_usage()?;

    println!("  Input tokens:      {:>12}", metrics.total_input_tokens);
    println!("  Output tokens:     {:>12}", metrics.total_output_tokens);
    println!(
        "  Reasoning tokens:  {:>12}",
        metrics.total_reasoning_tokens
    );
    println!(
        "  Cache write:       {:>12}",
        metrics.total_cache_write_tokens
    );
    println!(
        "  Cache read:        {:>12}",
        metrics.total_cache_read_tokens
    );
    println!("  Total cost:        {:>12.4}", metrics.total_cost);
    println!("  Interactions:      {:>12}", metrics.interaction_count);

    // Save to database
    let current_date = chrono::Utc::now().date_naive();
    println!("\n💾 Saving snapshot for {}...", current_date);

    repo.save_snapshot(current_date, &metrics)?;

    println!("✅ Snapshot saved successfully!");
    println!("\nThe applet and viewer should now display today's data.");

    Ok(())
}
