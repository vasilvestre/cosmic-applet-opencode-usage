// SPDX-License-Identifier: GPL-3.0-only

use cosmic_applet_template::app::CopilotMonitorApplet;
use cosmic_applet_template::core::config::AppConfig;

/// The `cosmic::app::run()` function is the starting point of your application.
/// It takes the Application type and flags (configuration) to initialize the applet.
/// The flags are passed to the Application::init method.
fn main() -> cosmic::iced::Result {
    // TODO: Load configuration from file or environment
    // For now, use default values
    let config = AppConfig {
        organization_name: "default-org".to_string(),
        refresh_interval_seconds: 900, // 15 minutes
    };
    
    cosmic::applet::run::<CopilotMonitorApplet>(config)
}
