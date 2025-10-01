// SPDX-License-Identifier: GPL-3.0-only

//! Main entry point for the `OpenCode` Usage History viewer application.

use cosmic_applet_opencode_usage::viewer::ViewerApp;

fn main() -> cosmic::iced::Result {
    // Configure window settings
    let settings = cosmic::app::Settings::default().size(cosmic::iced::Size::new(1000.0, 700.0));

    // Run the application
    cosmic::app::run::<ViewerApp>(settings, ())
}
