// SPDX-License-Identifier: GPL-3.0-only

//! Backfill historical daily snapshots from `OpenCode` storage.
//!
//! This tool analyzes file modification times and creates daily snapshots
//! for past dates, allowing historical data visualization.

use chrono::{NaiveDate, TimeZone};
use cosmic_applet_opencode_usage::core::{
    database::DatabaseManager,
    opencode::{OpenCodeUsageReader, StorageScanner, UsageAggregator, UsageParser},
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Historical Data Backfill Tool\n");

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

    // Create scanner with default OpenCode storage path
    let storage_path = std::path::PathBuf::from(&home).join(".local/share/opencode/storage/part");
    println!("Storage:  {}\n", storage_path.display());

    if !storage_path.exists() {
        eprintln!("❌ OpenCode storage directory not found!");
        eprintln!("Expected: {}", storage_path.display());
        return Ok(());
    }

    let _reader = OpenCodeUsageReader::new()?;

    // Scan all files with metadata
    println!("📊 Scanning all files...");
    let scanner = StorageScanner::with_path(storage_path)?;
    let all_files = scanner.scan_with_metadata()?;
    println!("  Found {} files total\n", all_files.len());

    // Group files by date (based on modification time)
    println!("📅 Grouping files by date...");
    let mut files_by_date: BTreeMap<NaiveDate, Vec<_>> = BTreeMap::new();

    for file_meta in &all_files {
        // Convert SystemTime to NaiveDate
        let duration_since_epoch = file_meta
            .modified
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        // Convert u64 seconds to i64 for chrono, clamping to i64::MAX to prevent wrap
        #[allow(clippy::cast_possible_wrap)] // Clamped to i64::MAX, safe for chrono
        let timestamp_secs = duration_since_epoch.as_secs().min(i64::MAX as u64) as i64;

        let datetime = chrono::Utc.timestamp_opt(timestamp_secs, 0).single();
        if let Some(dt) = datetime {
            let date = dt.date_naive();
            files_by_date.entry(date).or_default().push(file_meta);
        }
    }

    println!("  Found data spanning {} days\n", files_by_date.len());

    if files_by_date.is_empty() {
        println!("✅ No historical data to backfill");
        return Ok(());
    }

    // Show date range
    let first_date = files_by_date.keys().next().unwrap();
    let last_date = files_by_date.keys().last().unwrap();
    println!(
        "📆 Date range: {} to {} ({} days)",
        first_date,
        last_date,
        (*last_date - *first_date).num_days() + 1
    );

    // Ask for confirmation
    println!("\n⚠️  This will create daily snapshots for each day with data.");
    println!("Each snapshot will contain CUMULATIVE usage up to that day.");
    println!("\nDo you want to continue? (yes/no)");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    println!("\n🔄 Creating daily snapshots...\n");

    // Process each date
    let mut saved_count = 0;
    let total_dates = files_by_date.len();

    for (idx, (date, files)) in files_by_date.iter().enumerate() {
        // Check if snapshot already exists
        if repo.get_snapshot(*date)?.is_some() {
            println!(
                "[{:>4}/{:>4}] {} - ⏭️  Snapshot already exists (skipping)",
                idx + 1,
                total_dates,
                date
            );
            continue;
        }

        // Parse and aggregate files for this date and all previous dates
        // This gives us cumulative usage up to this date
        // Reuse already-collected all_files instead of rescanning
        let end_of_day_timestamp = date.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp();

        // Only convert to u64 if positive (dates before 1970 would be negative)
        #[allow(clippy::cast_sign_loss)] // Already checked for negative values
        let cutoff = if end_of_day_timestamp >= 0 {
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(end_of_day_timestamp as u64)
        } else {
            SystemTime::UNIX_EPOCH
        };

        let relevant_files: Vec<_> = all_files.iter().filter(|f| f.modified <= cutoff).collect();

        // Parse and aggregate
        let mut aggregator = UsageAggregator::new();
        for file_meta in relevant_files {
            if let Ok(Some(part)) = UsageParser::parse_file(&file_meta.path) {
                aggregator.add_part(&part);
            }
        }
        let metrics = aggregator.finalize();

        // Save snapshot
        repo.save_snapshot(*date, &metrics)?;
        saved_count += 1;

        println!(
            "[{:>4}/{:>4}] {} - ✅ Saved ({} files, {} interactions)",
            idx + 1,
            total_dates,
            date,
            files.len(),
            metrics.interaction_count
        );
    }

    println!("\n✅ Backfill complete! Created {saved_count} new snapshots.\n");
    println!("The viewer should now display historical data.");

    Ok(())
}
