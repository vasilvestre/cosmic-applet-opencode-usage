// SPDX-License-Identifier: GPL-3.0-only

use cosmic_applet_opencode_usage::app::OpenCodeMonitorApplet;
use cosmic_applet_opencode_usage::core::config::AppConfig;

fn main() -> cosmic::iced::Result {
    // Load config from COSMIC config system, fall back to defaults if not found
    let config = AppConfig::load().unwrap_or_else(|err| {
        eprintln!("Warning: Failed to load config ({}), using defaults", err);
        AppConfig::default()
    });

    cosmic::applet::run::<OpenCodeMonitorApplet>(config)
}
