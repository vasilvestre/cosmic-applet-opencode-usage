// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    Element, Application,
    app::{Core, Task},
    iced::{
        window,
        platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup},
    },
    widget::{button, checkbox, column, container, row, scrollable, text, text_input},
};

use crate::core::config::{AppConfig, ConfigError, ConfigWarning, validate_refresh_interval};
use crate::core::opencode::OpenCodeUsageReader;
use crate::ui::state::{AppState, PanelState, DisplayMode};
use crate::ui::Message;

/// OpenCode usage monitor applet structure
pub struct OpenCodeMonitorApplet {
    /// Application state managed by COSMIC runtime
    core: Core,
    /// Application state containing UI and data state
    state: AppState,
    /// OpenCode usage reader
    reader: OpenCodeUsageReader,
    /// Settings UI state
    settings_dialog_open: bool,
    temp_refresh_interval: u32,
    temp_refresh_interval_str: String,
    temp_show_today_usage: bool,
    config_error: Option<ConfigError>,
    config_warning: Option<ConfigWarning>,
    /// Popup window tracking
    popup: Option<cosmic::iced::window::Id>,
}

impl OpenCodeMonitorApplet {
    /// Create a new OpenCodeMonitorApplet instance
    pub fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = if let Some(ref path) = config.storage_path {
            OpenCodeUsageReader::new_with_path(path.to_str().ok_or("Invalid storage path")?)?
        } else {
            OpenCodeUsageReader::new()?
        };
        
        let temp_refresh_interval = config.refresh_interval_seconds;
        let temp_show_today_usage = config.show_today_usage;
        
        Ok(Self {
            core: Core::default(),
            state: AppState::new(config),
            reader,
            settings_dialog_open: false,
            temp_refresh_interval,
            temp_refresh_interval_str: temp_refresh_interval.to_string(),
            temp_show_today_usage,
            config_error: None,
            config_warning: None,
            popup: None,
        })
    }

    /// Handle incoming messages and update application state
    pub fn handle_message(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchMetrics => {
                eprintln!("[FetchMetrics] Starting OpenCode usage read (mode: {:?})", self.state.display_mode);
                
                // Read metrics based on display mode
                let result = match self.state.display_mode {
                    DisplayMode::Today => {
                        eprintln!("[FetchMetrics] Fetching today's usage only");
                        self.reader.get_usage_today()
                    }
                    DisplayMode::AllTime => {
                        eprintln!("[FetchMetrics] Fetching all-time usage");
                        self.reader.get_usage()
                    }
                };
                
                match result {
                    Ok(metrics) => {
                        eprintln!("[FetchMetrics] Successfully read {} interactions", metrics.interaction_count);
                        self.state.update_success(metrics);
                    }
                    Err(e) => {
                        eprintln!("[FetchMetrics] Error reading metrics: {}", e);
                        let error_msg = format!("Failed to read OpenCode usage: {}", e);
                        self.state.update_error(error_msg);
                    }
                }

                // If show_today_usage is enabled, also fetch today's usage for panel display
                if self.state.config.show_today_usage {
                    eprintln!("[FetchMetrics] Fetching today's usage for panel display");
                    match self.reader.get_usage_today() {
                        Ok(today_metrics) => {
                            eprintln!("[FetchMetrics] Successfully cached today's usage for panel: ${:.2}", today_metrics.total_cost);
                            self.state.update_today_usage(today_metrics);
                        }
                        Err(e) => {
                            eprintln!("[FetchMetrics] Error fetching today's usage for panel: {}", e);
                            self.state.clear_today_usage();
                        }
                    }
                }

                Task::none()
            }
            Message::MetricsFetched(Ok(usage)) => {
                eprintln!("[MetricsFetched] Received successful metrics data");
                self.state.update_success(usage);
                Task::none()
            }
            Message::MetricsFetched(Err(error)) => {
                eprintln!("[MetricsFetched] Received error: {}", error);
                self.state.update_error(error);
                Task::none()
            }
            Message::ThemeChanged => Task::none(),
            Message::UpdateTooltip => Task::none(),
            Message::OpenSettings => {
                self.settings_dialog_open = true;
                self.temp_refresh_interval = self.state.config.refresh_interval_seconds;
                self.temp_refresh_interval_str = self.temp_refresh_interval.to_string();
                self.temp_show_today_usage = self.state.config.show_today_usage;
                self.config_error = None;
                self.config_warning = None;
                Task::none()
            }
            Message::CloseSettings => {
                self.settings_dialog_open = false;
                self.config_error = None;
                self.config_warning = None;
                Task::none()
            }
            Message::UpdateRefreshInterval(interval) => {
                self.temp_refresh_interval = interval;
                self.temp_refresh_interval_str = interval.to_string();
                Task::none()
            }
            Message::ToggleShowTodayUsage(enabled) => {
                self.temp_show_today_usage = enabled;
                Task::none()
            }
            Message::ToggleDisplayMode => {
                eprintln!("[ToggleDisplayMode] Switching from {:?} to {:?}", 
                    self.state.display_mode, self.state.display_mode.toggle());
                self.state.toggle_display_mode();
                // Trigger a refresh to fetch data for the new mode
                Task::done(cosmic::Action::App(Message::FetchMetrics))
            }
            Message::SaveConfig => {
                // Validate refresh interval
                match validate_refresh_interval(self.temp_refresh_interval) {
                    Err(err) => {
                        // Hard error - don't save
                        self.config_error = Some(err);
                        self.config_warning = None;
                        return Task::none();
                    }
                    Ok(warning) => {
                        // Valid (with optional warning) - save the config
                        self.config_error = None;
                        self.config_warning = warning;
                    }
                }

                // Update config in state (no persistence for now - will be added later)
                self.state.config.refresh_interval_seconds = self.temp_refresh_interval;
                self.state.config.show_today_usage = self.temp_show_today_usage;
                
                // Clear today's usage cache if the setting was disabled
                if !self.temp_show_today_usage {
                    self.state.clear_today_usage();
                }
                
                // Success: close settings
                self.settings_dialog_open = false;
                self.popup = None;

                Task::none()
            }
            Message::TogglePopup => {
                eprintln!("DEBUG: TogglePopup message received");
                if let Some(p) = self.popup.take() {
                    eprintln!("DEBUG: Closing popup with id: {:?}", p);
                    self.settings_dialog_open = false;
                    self.config_error = None;
                    self.config_warning = None;
                    return destroy_popup(p);
                } else {
                    eprintln!("DEBUG: Opening popup");
                    let new_id = window::Id::unique();
                    eprintln!("DEBUG: Created new popup id: {:?}", new_id);
                    self.popup.replace(new_id);

                    if let Some(main_id) = self.core.main_window_id() {
                        eprintln!("DEBUG: Got main window id: {:?}", main_id);
                        let popup_settings = self.core.applet.get_popup_settings(
                            main_id,
                            new_id,
                            None,
                            None,
                            None,
                        );
                        eprintln!("DEBUG: Created popup settings, calling get_popup");
                        return get_popup(popup_settings);
                    } else {
                        eprintln!("DEBUG: No main window ID - returning Task::none()");
                        return Task::none();
                    }
                }
            }
            Message::None => Task::none(),
        }
    }

    /// Get the tooltip text to display
    fn get_tooltip_text(&self) -> String {
        use crate::ui::formatters::format_tooltip;
        format_tooltip(self.state.last_update)
    }

    /// Get the icon name based on current state
    fn get_state_icon(&self) -> &'static str {
        match &self.state.panel_state {
            PanelState::Loading => "content-loading-symbolic",
            PanelState::Error(_) => "dialog-error-symbolic",
            PanelState::Success(_) => "dialog-information-symbolic",
            PanelState::Stale(_) => "dialog-information-symbolic",
        }
    }

    /// Build the metrics popup view
    fn metrics_popup_view(&self) -> Element<'_, Message> {
        use crate::ui::formatters::{format_number, format_cost, format_tooltip};
        
        let main_content = match &self.state.panel_state {
            PanelState::Loading => {
                column()
                    .push(text("Loading...").size(16))
                    .push(text("").size(8))
                    .push(button::standard("Settings").on_press(Message::OpenSettings))
                    .spacing(10)
                    .padding(20)
            }
            PanelState::Error(err) => {
                column()
                    .push(text("Error").size(20))
                    .push(text(err).size(14))
                    .push(text("").size(8))
                    .push(button::standard("Retry").on_press(Message::FetchMetrics))
                    .push(button::standard("Settings").on_press(Message::OpenSettings))
                    .spacing(10)
                    .padding(20)
            }
            PanelState::Success(usage) | PanelState::Stale(usage) => {
                // Determine button label based on current mode
                let toggle_button_text = match self.state.display_mode {
                    DisplayMode::Today => "Show All Time",
                    DisplayMode::AllTime => "Show Today",
                };
                
                // Determine title based on current mode
                let title = match self.state.display_mode {
                    DisplayMode::Today => "Today's Usage",
                    DisplayMode::AllTime => "All-Time Usage",
                };
                
                column()
                    .push(text(title).size(20))
                    .push(text("").size(4))
                    .push(button::standard(toggle_button_text).on_press(Message::ToggleDisplayMode))
                    .push(text("").size(8))
                    .push(row()
                        .push(text("Total Cost: ").size(14))
                        .push(text(format_cost(usage.total_cost)).size(14))
                        .spacing(5)
                    )
                    .push(row()
                        .push(text("Interactions: ").size(14))
                        .push(text(format_number(usage.interaction_count as u64)).size(14))
                        .spacing(5)
                    )
                    .push(row()
                        .push(text("Input Tokens: ").size(14))
                        .push(text(format_number(usage.total_input_tokens)).size(14))
                        .spacing(5)
                    )
                    .push(row()
                        .push(text("Output Tokens: ").size(14))
                        .push(text(format_number(usage.total_output_tokens)).size(14))
                        .spacing(5)
                    )
                    .push(text("").size(8))
                    .push(text(format_tooltip(self.state.last_update)).size(12))
                    .push(text("").size(8))
                    .push(button::standard("Settings").on_press(Message::OpenSettings))
                    .spacing(10)
                    .padding(20)
            }
        };

        container(main_content).into()
    }

    /// Build the settings dialog UI
    fn settings_view(&self) -> Element<'_, Message> {
        let mut content = column()
            .push(text("OpenCode Monitor Settings").size(24))
            .push(text("").size(8))
            .push(text("Refresh Interval (seconds)").size(14))
            .push(
                text_input(
                    "Enter refresh interval",
                    &self.temp_refresh_interval_str
                )
                .on_input(|s| {
                    s.parse::<u32>()
                        .map(Message::UpdateRefreshInterval)
                        .unwrap_or(Message::None)
                })
            )
            .push(text("").size(8))
            .push(text("Display Options").size(14))
            .push(
                checkbox(
                    "Show today's usage next to icon",
                    self.temp_show_today_usage
                )
                .on_toggle(Message::ToggleShowTodayUsage)
            )
            .spacing(10)
            .padding(20);
        
        // Show error if present (red/critical style)
        if let Some(ref err) = self.config_error {
            content = content
                .push(text("").size(8))
                .push(text(format!("❌ Error: {}", err)).size(14));
        }
        
        // Show warning if present (yellow/info style)
        if let Some(ref warn) = self.config_warning {
            let warning_text = match warn {
                ConfigWarning::LowRefreshInterval(interval) => {
                    format!("⚠️  Warning: Refresh interval of {} seconds is very low. This may cause high CPU usage and frequent file system access.", interval)
                }
            };
            content = content
                .push(text("").size(8))
                .push(text(warning_text).size(14));
        }
        
        // Add action buttons
        content = content
            .push(text("").size(12))
            .push(
                row()
                    .push(button::standard("Cancel").on_press(Message::CloseSettings))
                    .push(button::suggested("Save").on_press(Message::SaveConfig))
                    .spacing(12)
            );
        
        scrollable(content).into()
    }
}

/// Implement the Application trait for OpenCodeMonitorApplet
impl Application for OpenCodeMonitorApplet {
    type Executor = cosmic::executor::Default;
    type Flags = AppConfig;
    type Message = Message;
    const APP_ID: &'static str = "com.system76.CosmicAppletOpenCodeMonitor";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let reader = if let Some(ref path) = flags.storage_path {
            OpenCodeUsageReader::new_with_path(path.to_str().unwrap_or("")).expect("Failed to create OpenCode reader")
        } else {
            OpenCodeUsageReader::new().expect("Failed to create OpenCode reader")
        };
        
        let temp_refresh_interval = flags.refresh_interval_seconds;
        let temp_show_today_usage = flags.show_today_usage;
        
        let applet = Self {
            core,
            state: AppState::new(flags),
            reader,
            settings_dialog_open: false,
            temp_refresh_interval,
            temp_refresh_interval_str: temp_refresh_interval.to_string(),
            temp_show_today_usage,
            config_error: None,
            config_warning: None,
            popup: None,
        };

        eprintln!("[init] Application initialized, triggering initial FetchMetrics");
        (applet, Task::done(cosmic::Action::App(Message::FetchMetrics)))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        use crate::ui::formatters::format_cost_compact;
        
        // If show_today_usage is enabled and we have today's data, show cost text as button
        if self.state.config.show_today_usage {
            if let Some(today_usage) = &self.state.today_usage {
                let cost_text = format_cost_compact(today_usage.total_cost);
                return container(
                    button::standard(cost_text)
                        .on_press(Message::TogglePopup)
                )
                .into();
            }
        }
        
        // Default: just show icon
        self.core
            .applet
            .icon_button(self.get_state_icon())
            .on_press_down(Message::TogglePopup)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        self.handle_message(message)
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
        if self.popup.is_some() && self.popup == Some(id) {
            let content = if self.settings_dialog_open {
                self.settings_view()
            } else {
                self.metrics_popup_view()
            };
            
            let (max_w, max_h) = if self.settings_dialog_open {
                (600.0, 600.0)
            } else {
                (500.0, 400.0)
            };
            
            self.core
                .applet
                .popup_container(content)
                .max_width(max_w)
                .max_height(max_h)
                .into()
        } else {
            text("").into()
        }
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        if self.popup == Some(id) {
            Some(Message::TogglePopup)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::AppConfig;
    use crate::core::opencode::UsageMetrics;
    use crate::ui::state::PanelState;
    use std::time::SystemTime;

    fn create_mock_config() -> AppConfig {
        AppConfig {
            storage_path: None,
            refresh_interval_seconds: 900,
            show_today_usage: false,
        }
    }

    fn create_mock_usage_metrics() -> UsageMetrics {
        UsageMetrics {
            total_input_tokens: 1000,
            total_output_tokens: 500,
            total_reasoning_tokens: 200,
            total_cache_write_tokens: 100,
            total_cache_read_tokens: 50,
            total_cost: 12.50,
            interaction_count: 10,
            timestamp: SystemTime::now(),
        }
    }

    #[test]
    fn test_applet_initialization() {
        let config = create_mock_config();
        let applet = OpenCodeMonitorApplet::new(config);
        // May fail if OpenCode directory doesn't exist, which is OK for this test
        if let Ok(applet) = applet {
            assert!(matches!(applet.state.panel_state, PanelState::Loading));
        }
    }

    #[test]
    fn test_handle_metrics_fetched_success() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            let usage = create_mock_usage_metrics();
            
            applet.handle_message(Message::MetricsFetched(Ok(usage.clone())));
            
            assert!(matches!(applet.state.panel_state, PanelState::Success(_)));
            assert!(applet.state.last_update.is_some());
        }
    }

    #[test]
    fn test_handle_metrics_fetched_error() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            let error = "Test error".to_string();
            
            applet.handle_message(Message::MetricsFetched(Err(error)));
            
            assert!(matches!(applet.state.panel_state, PanelState::Error(_)));
        }
    }

    #[test]
    fn test_settings_operations() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Open settings
            applet.handle_message(Message::OpenSettings);
            assert!(applet.settings_dialog_open);
            
            // Update refresh interval
            applet.handle_message(Message::UpdateRefreshInterval(1800));
            assert_eq!(applet.temp_refresh_interval, 1800);
            
            // Close settings
            applet.handle_message(Message::CloseSettings);
            assert!(!applet.settings_dialog_open);
        }
    }

    #[test]
    fn test_toggle_display_mode() {
        use crate::ui::state::DisplayMode;
        
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Should start with AllTime mode
            assert_eq!(applet.state.display_mode, DisplayMode::AllTime);
            
            // Toggle to Today mode
            let _ = applet.handle_message(Message::ToggleDisplayMode);
            assert_eq!(applet.state.display_mode, DisplayMode::Today);
            
            // Toggle back to AllTime
            let _ = applet.handle_message(Message::ToggleDisplayMode);
            assert_eq!(applet.state.display_mode, DisplayMode::AllTime);
        }
    }

    #[test]
    fn test_show_today_usage_toggle() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Should start disabled
            assert!(!applet.state.config.show_today_usage);
            assert!(!applet.temp_show_today_usage);
            
            // Open settings
            let _ = applet.handle_message(Message::OpenSettings);
            assert!(applet.settings_dialog_open);
            
            // Toggle show_today_usage
            let _ = applet.handle_message(Message::ToggleShowTodayUsage(true));
            assert!(applet.temp_show_today_usage);
            
            // Save config
            let _ = applet.handle_message(Message::SaveConfig);
            assert!(applet.state.config.show_today_usage);
            assert!(!applet.settings_dialog_open);
        }
    }

    #[test]
    fn test_show_today_usage_clears_cache_when_disabled() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Add some today usage data
            let usage = create_mock_usage_metrics();
            applet.state.update_today_usage(usage);
            assert!(applet.state.today_usage.is_some());
            
            // Open settings and toggle off
            let _ = applet.handle_message(Message::OpenSettings);
            let _ = applet.handle_message(Message::ToggleShowTodayUsage(false));
            let _ = applet.handle_message(Message::SaveConfig);
            
            // Cache should be cleared
            assert!(applet.state.today_usage.is_none());
        }
    }
}
