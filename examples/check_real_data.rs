// SPDX-License-Identifier: GPL-3.0-only

//! Quick tool to check what's actually in the production database.

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

    // Get total count
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM usage_snapshots", [], |row| row.get(0))?;
    println!("\nTotal snapshots: {count}");

    // Get all data ordered by date
    println!("\nAll snapshots in database:");
    println!("{:-<100}", "");

    let mut stmt = conn.prepare(
        "SELECT date, input_tokens, output_tokens, reasoning_tokens, 
                cache_write_tokens, cache_read_tokens, total_cost, interaction_count
         FROM usage_snapshots 
         ORDER BY date DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, f64>(6)?,
            row.get::<_, i32>(7)?,
        ))
    })?;

    for row in rows {
        let (date, input, output, reasoning, cache_w, cache_r, cost, interactions) = row?;
        println!("Date: {date}");
        println!("  Input: {input:>10}  Output: {output:>10}  Reasoning: {reasoning:>10}");
        println!("  Cache Write: {cache_w:>6}  Cache Read: {cache_r:>6}");
        println!("  Cost: ${cost:>8.4}  Interactions: {interactions:>4}");
        println!();
    }

    Ok(())
}
