// SPDX-License-Identifier: GPL-3.0-only

//! Example demonstrating basic database usage.
//!
//! This example shows how to:
//! - Initialize a database
//! - Insert usage snapshots
//! - Query data
//!
//! Run with: `cargo run --example database_usage`

use cosmic_applet_opencode_usage::core::database::DatabaseManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Database Usage Example");
    println!("======================\n");

    // Create a temporary database for this example
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("example.db");

    println!("1. Initializing database at: {}", db_path.display());
    let manager = DatabaseManager::new_with_path(&db_path)?;
    println!("   ✓ Database initialized successfully\n");

    // Insert a sample usage snapshot
    println!("2. Inserting usage snapshot...");
    {
        let conn = manager.get_connection();
        conn.execute(
            "INSERT INTO usage_snapshots 
             (date, input_tokens, output_tokens, reasoning_tokens, 
              cache_write_tokens, cache_read_tokens, total_cost, 
              interaction_count, created_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                "2024-01-15",
                15420, // input_tokens
                8234,  // output_tokens
                2145,  // reasoning_tokens
                1230,  // cache_write_tokens
                567,   // cache_read_tokens
                0.387, // total_cost
                42,    // interaction_count
                "2024-01-15T18:30:00Z"
            ],
        )?;
    }
    println!("   ✓ Snapshot inserted\n");

    // Query the data
    println!("3. Querying usage data...");
    {
        let conn = manager.get_connection();

        let mut stmt = conn.prepare(
            "SELECT date, input_tokens, output_tokens, total_cost, interaction_count 
             FROM usage_snapshots 
             ORDER BY date DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, i32>(4)?,
            ))
        })?;

        println!("   Usage Snapshots:");
        println!("   ----------------");
        for row in rows {
            let (date, input, output, cost, interactions) = row?;
            println!("   Date: {date}");
            println!("     Input Tokens:  {input:>8}");
            println!("     Output Tokens: {output:>8}");
            println!("     Total Cost:    ${cost:>7.3}");
            println!("     Interactions:  {interactions:>8}");
            println!();
        }
    }

    // Check schema version
    println!("4. Checking schema version...");
    {
        let conn = manager.get_connection();
        let version: i32 =
            conn.query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                row.get(0)
            })?;
        println!("   Current schema version: {version}");
    }

    println!("\n✓ Example completed successfully!");

    Ok(())
}
