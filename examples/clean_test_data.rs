// SPDX-License-Identifier: GPL-3.0-only

//! Clean test data from the database (snapshots with `input_tokens` = 1000).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rusqlite::Connection;
    use std::path::PathBuf;

    // Manually construct the path
    let home = std::env::var("HOME")?;
    let db_path = PathBuf::from(home).join(".local/share/cosmic-applet-opencode-usage/usage.db");

    println!("Database: {}", db_path.display());

    if !db_path.exists() {
        println!("Database doesn't exist!");
        return Ok(());
    }

    let conn = Connection::open(&db_path)?;

    // Check how many test snapshots exist
    let test_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM usage_snapshots WHERE input_tokens = 1000",
        [],
        |row| row.get(0),
    )?;

    let total_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM usage_snapshots", [], |row| row.get(0))?;

    println!("\nBefore cleanup:");
    println!("  Total snapshots: {total_count}");
    println!("  Test snapshots (input_tokens = 1000): {test_count}");

    if test_count == 0 {
        println!("\nNo test data to clean!");
        return Ok(());
    }

    // Delete test data
    let deleted = conn.execute("DELETE FROM usage_snapshots WHERE input_tokens = 1000", [])?;

    let remaining: i64 =
        conn.query_row("SELECT COUNT(*) FROM usage_snapshots", [], |row| row.get(0))?;

    println!("\nAfter cleanup:");
    println!("  Deleted: {deleted} test snapshots");
    println!("  Remaining: {remaining} real snapshots");

    Ok(())
}
