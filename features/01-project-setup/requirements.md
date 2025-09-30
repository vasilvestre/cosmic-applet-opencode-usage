# Feature: Project Setup & Boilerplate Applet

## Overview
Establish the foundational Rust project structure and create a minimal COSMIC applet that successfully loads into the COSMIC panel. This validates basic integration with the COSMIC desktop environment using the official cosmic-applet template.

## Requirements

### Project Structure
- The system SHALL create a Rust project with edition 2021 and rust-version 1.80
- The system SHALL include libcosmic as a git dependency from the official pop-os repository
- The system SHALL configure libcosmic with features: "applet", "tokio", and "wayland"
- The system SHALL include i18n-embed and i18n-embed-fl for internationalization support
- The system SHALL include rust-embed for resource embedding
- The system SHALL organize code into modules: app, core, and main
- The system SHALL include a GPL-3.0 license

### Application Structure
- The system SHALL implement the cosmic::Application trait for the main applet struct
- The system SHALL define a Core field managed by the COSMIC runtime
- The system SHALL define a Message enum for application message passing
- The system SHALL set APP_ID to a unique identifier following reverse domain notation
- The system SHALL use cosmic::executor::Default as the Executor type
- The system SHALL implement init(), view(), view_window(), and update() methods

### Minimal Applet Integration
- WHEN the applet is compiled THEN the system SHALL produce a binary using cosmic::applet::run()
- The system SHALL display an icon button in the COSMIC panel using core.applet.icon_button()
- WHEN the user clicks the applet icon THEN the system SHALL toggle a popup window
- The system SHALL use core.applet.popup_container() for popup window content
- The system SHALL handle popup lifecycle with popup IDs and close events
- The system SHALL use libcosmic components for all UI elements

### Internationalization
- The system SHALL include i18n configuration in i18n.toml
- The system SHALL provide Fluent translation files in i18n/ directory
- The system SHALL use fl! macro for translatable strings
- The system SHALL include core/localization.rs module for i18n support

### Resources and Metadata
- The system SHALL include desktop entry file in res/ directory
- The system SHALL include metainfo.xml for AppStream metadata
- The system SHALL provide icon files in multiple sizes (16x16, 24x24, 32x32, 48x48, 64x64, 128x128, 256x256)
- The system SHALL use SVG format for scalable icons

### Build and Development
- The system SHALL compile successfully on Pop!_OS 24.04+
- The system SHALL provide clear build instructions in a README
- WHEN cargo build is executed THEN the system SHALL complete without errors
- The system SHALL support cargo run for local testing
- The system SHALL support justfile for common development tasks

## Acceptance Criteria
- [ ] Cargo.toml configured with libcosmic, i18n-embed, i18n-embed-fl, rust-embed
- [ ] src/main.rs calls cosmic::applet::run() with application struct
- [ ] src/app.rs implements Application trait with all required methods
- [ ] src/core/mod.rs and localization.rs modules exist
- [ ] Icon button appears in COSMIC panel
- [ ] Clicking icon button toggles popup window
- [ ] Popup window displays with proper container styling
- [ ] i18n.toml and translation files present
- [ ] Desktop entry and metainfo.xml configured
- [ ] Icon files present in all required sizes
- [ ] README with build instructions exists
- [ ] justfile with development commands exists

## Technical Notes
- Reference: Official cosmic-applet-template repository
- Use cosmic::applet::run() as entry point (not cosmic::app::run())
- Popup management uses cosmic::iced_winit::commands::popup::{get_popup, destroy_popup}
- Icon button syntax: core.applet.icon_button("icon-name").on_press(Message)
- Window IDs use cosmic::iced::window::Id::unique()
- Popup settings configured via core.applet.get_popup_settings()
- Style method should return Some(cosmic::applet::style())

## Dependencies
- None (foundational feature)
