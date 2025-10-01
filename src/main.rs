// SPDX-License-Identifier: GPL-3.0-only

use cosmic_applet_opencode_usage::app::OpenCodeMonitorApplet;
use cosmic_applet_opencode_usage::core::config::AppConfig;

fn main() -> cosmic::iced::Result {
    // Use default configuration for now (will add persistence later)
    let config = AppConfig::default();

    cosmic::applet::run::<OpenCodeMonitorApplet>(config)
}
