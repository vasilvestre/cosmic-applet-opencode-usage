// SPDX-License-Identifier: GPL-3.0-only

//! Apply pending database migrations.

use cosmic_applet_opencode_usage::core::database::DatabaseManager;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Database Migration Tool\n");

    // Use the production database path
    let home = std::env::var("HOME")?;
    let db_path = PathBuf::from(home).join(".local/share/cosmic-applet-opencode-usage/usage.db");

    println!("Database path: {}", db_path.display());

    if !db_path.exists() {
        println!("❌ Database doesn't exist!");
        return Ok(());
    }

    println!("\n🔄 Applying pending migrations...");

    // DatabaseManager will automatically run migrations on creation
    let _db = DatabaseManager::new_with_path(&db_path)?;

    println!("✅ Migrations applied successfully!");
    println!("\nThe database now has:");
    println!("  - UNIQUE constraint on date column");
    println!("  - Duplicates automatically removed (keeping most recent)");

    Ok(())
}
