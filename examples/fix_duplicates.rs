// SPDX-License-Identifier: GPL-3.0-only

//! Fix duplicate date entries in the database.

use rusqlite::Connection;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking for duplicate dates in database...\n");

    // Use the production database path
    let home = std::env::var("HOME")?;
    let db_path = PathBuf::from(home).join(".local/share/cosmic-applet-opencode-usage/usage.db");

    println!("Database path: {}", db_path.display());

    if !db_path.exists() {
        println!("❌ Database doesn't exist!");
        return Ok(());
    }

    let conn = Connection::open(&db_path)?;

    // Check for duplicates
    println!("\n🔍 Checking for duplicate dates...");
    let mut stmt = conn.prepare(
        "SELECT date, COUNT(*) as count, SUM(input_tokens) as total_input, 
                SUM(output_tokens) as total_output
         FROM usage_snapshots 
         GROUP BY date 
         HAVING COUNT(*) > 1
         ORDER BY date DESC",
    )?;

    let duplicates: Vec<(String, i32)> = stmt
        .query_map([], |row| {
            let date: String = row.get(0)?;
            let count: i32 = row.get(1)?;
            let total_input: i64 = row.get(2)?;
            let total_output: i64 = row.get(3)?;
            println!(
                "  ❌ Date {} has {} entries (total input: {}, output: {})",
                date, count, total_input, total_output
            );
            Ok((date, count))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    if duplicates.is_empty() {
        println!("  ✅ No duplicates found!");
        return Ok(());
    }

    println!("\n⚠️  Found {} dates with duplicates", duplicates.len());
    println!("\nDo you want to remove duplicates? (yes/no)");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    // For each duplicate date, keep only the most recent entry
    println!("\n🔧 Removing duplicates (keeping most recent entry for each date)...");

    for (date, _count) in duplicates {
        conn.execute(
            "DELETE FROM usage_snapshots 
             WHERE date = ?1 
             AND id NOT IN (
                 SELECT id FROM usage_snapshots 
                 WHERE date = ?1 
                 ORDER BY created_at DESC 
                 LIMIT 1
             )",
            [&date],
        )?;
        println!("  ✅ Cleaned up duplicates for {}", date);
    }

    println!("\n✨ Done! Database cleaned.");

    // Show summary
    let total_snapshots: i32 =
        conn.query_row("SELECT COUNT(*) FROM usage_snapshots", [], |row| row.get(0))?;

    println!("\n📊 Total snapshots remaining: {}", total_snapshots);

    Ok(())
}
