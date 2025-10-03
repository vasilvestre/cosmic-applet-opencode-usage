// SPDX-License-Identifier: GPL-3.0-only

//! Debug tool to check what's in the production database.

use cosmic_applet_opencode_usage::core::database::repository::UsageRepository;
use cosmic_applet_opencode_usage::core::database::DatabaseManager;
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking production database...\n");

    // Use the production database path
    let home = std::env::var("HOME")?;
    let db_path = PathBuf::from(home).join(".local/share/cosmic-applet-opencode-usage/usage.db");

    println!("Database path: {}", db_path.display());

    if !db_path.exists() {
        println!("❌ Database doesn't exist!");
        return Ok(());
    }

    // Create repository
    let db_manager = Arc::new(DatabaseManager::new_with_path(&db_path)?);
    let repository = UsageRepository::new(db_manager);

    // Get all snapshots to count and show recent ones
    let today = chrono::Utc::now().date_naive();
    let long_ago = today - chrono::Duration::days(365 * 2); // Get up to 2 years of data
    let all_snapshots = repository.get_range(long_ago, today)?;

    println!("\n📊 Total snapshots in database: {}", all_snapshots.len());

    // Show latest 5 snapshots
    println!("\n📋 Latest 5 snapshots:");
    println!("---------------------");
    let recent_snapshots: Vec<_> = all_snapshots.iter().rev().take(5).collect();
    for snapshot in &recent_snapshots {
        println!("  Date: {}", snapshot.date);
        println!("    Input:       {:>8}", snapshot.input_tokens);
        println!("    Output:      {:>8}", snapshot.output_tokens);
        println!("    Reasoning:   {:>8}", snapshot.reasoning_tokens);
        println!("    Cost:        ${:.4}", snapshot.total_cost);
        println!("    Interactions: {:>8}", snapshot.interaction_count);
        println!();
    }

    // Check date range
    if let Some(first) = all_snapshots.first() {
        if let Some(last) = all_snapshots.last() {
            println!("📅 Date range: {} to {}", first.date, last.date);
        }
    }

    // Try to get this week's summary
    println!("\n📈 Checking week summaries...");
    let today = chrono::Utc::now().date_naive();
    println!("  Today: {today}");

    let this_week = repository.get_week_summary(today)?;
    println!("\n  This week summary:");
    println!("    Input:       {:>8}", this_week.total_input_tokens);
    println!("    Output:      {:>8}", this_week.total_output_tokens);
    println!("    Reasoning:   {:>8}", this_week.total_reasoning_tokens);
    println!("    Cost:        ${:.4}", this_week.total_cost);
    println!("    Interactions: {:>8}", this_week.total_interactions);

    let last_week_date = today - chrono::Duration::days(7);
    let last_week = repository.get_week_summary(last_week_date)?;
    println!("\n  Last week summary:");
    println!("    Input:       {:>8}", last_week.total_input_tokens);
    println!("    Output:      {:>8}", last_week.total_output_tokens);
    println!("    Reasoning:   {:>8}", last_week.total_reasoning_tokens);
    println!("    Cost:        ${:.4}", last_week.total_cost);
    println!("    Interactions: {:>8}", last_week.total_interactions);

    Ok(())
}
