// SPDX-License-Identifier: GPL-3.0-only

//! Clean the entire database and start fresh.

use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  Database Cleanup Tool\n");

    // Use the production database path
    let home = std::env::var("HOME")?;
    let db_path = PathBuf::from(home).join(".local/share/cosmic-applet-opencode-usage/usage.db");

    println!("Database path: {}", db_path.display());

    if !db_path.exists() {
        println!("✅ Database doesn't exist - nothing to clean!");
        return Ok(());
    }

    println!("\n⚠️  WARNING: This will delete ALL data in the database!");
    println!("This includes any real usage data that has been collected.");
    println!("\nDo you want to continue? (yes/no)");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    println!("\n🗑️  Removing database file...");
    fs::remove_file(&db_path)?;
    
    println!("✅ Database removed successfully!");
    println!("\nThe database will be recreated on next scan or viewer launch.");
    println!("Run the data collection to populate with real usage data.");

    Ok(())
}
