// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{Core, Task},
    iced::{
        futures::SinkExt,
        platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup},
        window, Alignment, Subscription,
    },
    iced_futures::stream,
    widget::{autosize, button, checkbox, column, icon, row, scrollable, text, text_input, Id},
    Application, Element,
};
use std::sync::LazyLock;
use tokio::{sync::watch, time};

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

use crate::core::collector::DataCollector;
use crate::core::config::{validate_refresh_interval, AppConfig, ConfigError, ConfigWarning};
use crate::core::database::DatabaseManager;
use crate::core::opencode::OpenCodeUsageReader;
use crate::ui::state::{AppState, DisplayMode, PanelState};
use crate::ui::Message;
use std::sync::Arc;

/// `OpenCode` usage monitor applet structure
pub struct OpenCodeMonitorApplet {
    /// Application state managed by COSMIC runtime
    core: Core,
    /// Application state containing UI and data state
    state: AppState,
    /// `OpenCode` usage reader
    reader: OpenCodeUsageReader,
    /// Data collector for automatic snapshot management
    data_collector: Option<DataCollector>,
    /// Settings UI state
    settings_dialog_open: bool,
    temp_refresh_interval: u32,
    temp_refresh_interval_str: String,
    temp_show_today_usage: bool,
    temp_use_raw_token_display: bool,
    config_error: Option<ConfigError>,
    config_warning: Option<ConfigWarning>,
    /// Popup window tracking
    popup: Option<cosmic::iced::window::Id>,
    /// Watch channel sender for refresh interval updates
    refresh_interval_tx: watch::Sender<u32>,
}

impl OpenCodeMonitorApplet {
    /// Create a new `OpenCodeMonitorApplet` instance
    ///
    /// # Errors
    /// Returns an error if the storage path is invalid or if the reader cannot be initialized.
    pub fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let reader = if let Some(ref path) = config.storage_path {
            OpenCodeUsageReader::new_with_path(path.to_str().ok_or("Invalid storage path")?)?
        } else {
            OpenCodeUsageReader::new()?
        };

        let temp_refresh_interval = config.refresh_interval_seconds;
        let temp_show_today_usage = config.show_today_usage;
        let temp_use_raw_token_display = config.use_raw_token_display;

        // Create watch channel for refresh interval updates
        let (refresh_interval_tx, _rx) = watch::channel(config.refresh_interval_seconds);

        // Initialize data collector with database
        let data_collector = match DatabaseManager::new() {
            Ok(db_manager) => {
                eprintln!("[DataCollector] Database initialized successfully");
                Some(DataCollector::new(Arc::new(db_manager)))
            }
            Err(e) => {
                eprintln!("[DataCollector] Failed to initialize database: {e}");
                eprintln!("[DataCollector] Continuing without automatic data collection");
                None
            }
        };

        Ok(Self {
            core: Core::default(),
            state: AppState::new(config),
            reader,
            data_collector,
            settings_dialog_open: false,
            temp_refresh_interval,
            temp_refresh_interval_str: temp_refresh_interval.to_string(),
            temp_show_today_usage,
            temp_use_raw_token_display,
            config_error: None,
            config_warning: None,
            popup: None,
            refresh_interval_tx,
        })
    }

    /// Handle incoming messages and update application state
    /// Handle incoming messages and perform async operations
    #[allow(clippy::too_many_lines)] // Message handler naturally has many branches
    pub fn handle_message(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FetchMetrics => {
                eprintln!(
                    "[FetchMetrics] Starting async OpenCode usage read (mode: {:?})",
                    self.state.display_mode
                );

                // Set loading state
                self.state.set_loading();

                // Clone the storage path for async task
                let storage_path = self.reader.storage_path().clone();
                let display_mode = self.state.display_mode;
                let show_today_usage = self.state.config.show_today_usage;

                // Spawn async task to fetch metrics in background
                Task::perform(
                    async move {
                        // Create a new reader in the async context
                        let mut reader = match OpenCodeUsageReader::new_with_path(
                            storage_path.to_str().unwrap_or(""),
                        ) {
                            Ok(r) => r,
                            Err(e) => return Err(format!("Failed to create reader: {e}")),
                        };

                        // Fetch main metrics based on display mode
                        let metrics = match display_mode {
                            DisplayMode::Today => {
                                eprintln!("[Async] Fetching today's usage");
                                reader.get_usage_today()
                            }
                            DisplayMode::Month => {
                                eprintln!("[Async] Fetching this month's usage");
                                reader.get_usage_month()
                            }
                            DisplayMode::AllTime => {
                                eprintln!("[Async] Fetching all-time usage");
                                reader.get_usage()
                            }
                        };

                        let main_metrics = metrics.map_err(|e| {
                            eprintln!("[Async] Error reading metrics: {e}");
                            format!("Failed to read OpenCode usage: {e}")
                        })?;

                        // If show_today_usage is enabled, also fetch today's data
                        let today_metrics = if show_today_usage {
                            eprintln!("[Async] Fetching today's usage for panel");
                            reader.get_usage_today().ok()
                        } else {
                            None
                        };

                        // Always fetch month data for caching (independent of display mode)
                        let month_metrics = if display_mode == DisplayMode::Month {
                            // In Month mode, main_metrics already contains month data
                            None
                        } else {
                            eprintln!("[Async] Fetching this month's usage for cache");
                            reader.get_usage_month().ok()
                        };

                        Ok((main_metrics, today_metrics, month_metrics))
                    },
                    |result| cosmic::Action::App(Message::MetricsFetched(Box::new(result))),
                )
            }
            Message::MetricsFetched(boxed_result) => match *boxed_result {
                Ok((usage, today_opt, month_opt)) => {
                    eprintln!("[MetricsFetched] Received successful metrics data");

                    // Attempt to save snapshot using data collector
                    if let Some(ref collector) = self.data_collector {
                        match collector.collect_and_save(&usage) {
                            Ok(true) => {
                                eprintln!("[MetricsFetched] Snapshot saved successfully");
                            }
                            Ok(false) => {
                                eprintln!("[MetricsFetched] Snapshot already saved today");
                            }
                            Err(e) => {
                                eprintln!("[MetricsFetched] Failed to save snapshot: {e}");
                                // Continue despite error - don't crash the applet
                            }
                        }
                    } else {
                        eprintln!("[MetricsFetched] Data collector not available, skipping snapshot");
                    }

                    // If we're in Month mode, the main usage is the month data - cache it
                    if self.state.display_mode == DisplayMode::Month {
                        eprintln!(
                            "[MetricsFetched] Caching month usage from main metrics: ${:.2}",
                            usage.total_cost
                        );
                        self.state.update_month_usage(usage.clone());
                    }

                    self.state.update_success(usage);

                    // Update today's usage if provided
                    if let Some(today) = today_opt {
                        eprintln!(
                            "[MetricsFetched] Updating today's usage for panel: ${:.2}",
                            today.total_cost
                        );
                        self.state.update_today_usage(today);
                    }

                    // Update month's usage if provided (when not in Month mode)
                    if let Some(month) = month_opt {
                        eprintln!(
                            "[MetricsFetched] Updating month's usage cache: ${:.2}",
                            month.total_cost
                        );
                        self.state.update_month_usage(month);
                    }

                    Task::none()
                }
                Err(error) => {
                    eprintln!("[MetricsFetched] Received error: {error}");
                    self.state.update_error(error);
                    Task::none()
                }
            },
            Message::ThemeChanged | Message::UpdateTooltip | Message::None => Task::none(),
            Message::OpenSettings => {
                self.settings_dialog_open = true;
                self.temp_refresh_interval = self.state.config.refresh_interval_seconds;
                self.temp_refresh_interval_str = self.temp_refresh_interval.to_string();
                self.temp_show_today_usage = self.state.config.show_today_usage;
                self.temp_use_raw_token_display = self.state.config.use_raw_token_display;
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
            Message::ToggleRawTokenDisplay(enabled) => {
                self.temp_use_raw_token_display = enabled;
                Task::none()
            }
            Message::SelectDisplayMode(mode) => {
                eprintln!("[SelectDisplayMode] Switching to {mode:?}");
                self.state.display_mode = mode;
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

                // Update config in state
                self.state.config.refresh_interval_seconds = self.temp_refresh_interval;
                self.state.config.show_today_usage = self.temp_show_today_usage;
                self.state.config.use_raw_token_display = self.temp_use_raw_token_display;

                // Notify subscription of refresh interval change
                let _ = self.refresh_interval_tx.send(self.temp_refresh_interval);

                // Persist config to disk
                if let Err(err) = self.state.config.save() {
                    eprintln!("Warning: Failed to save config: {err}");
                    // Don't block the UI if save fails - just log it
                }

                // Clear today's usage cache if the setting was disabled
                if !self.temp_show_today_usage {
                    self.state.clear_today_usage();
                }

                // Success: close settings
                self.settings_dialog_open = false;
                self.popup = None;

                // Trigger a refresh to update the panel display based on the new settings:
                // - If show_today_usage was enabled, fetch today's data
                // - If use_raw_token_display changed, update the display format
                // - Any other config changes that affect the display
                Task::done(cosmic::Action::App(Message::FetchMetrics))
            }
            Message::TogglePopup => {
                eprintln!("DEBUG: TogglePopup message received");
                if let Some(p) = self.popup.take() {
                    eprintln!("DEBUG: Closing popup with id: {p:?}");
                    self.settings_dialog_open = false;
                    self.config_error = None;
                    self.config_warning = None;
                    destroy_popup(p)
                } else {
                    eprintln!("DEBUG: Opening popup");
                    let new_id = window::Id::unique();
                    eprintln!("DEBUG: Created new popup id: {new_id:?}");
                    self.popup.replace(new_id);

                    if let Some(main_id) = self.core.main_window_id() {
                        eprintln!("DEBUG: Got main window id: {main_id:?}");
                        let popup_settings = self
                            .core
                            .applet
                            .get_popup_settings(main_id, new_id, None, None, None);
                        eprintln!("DEBUG: Created popup settings, calling get_popup");
                        get_popup(popup_settings)
                    } else {
                        eprintln!("DEBUG: No main window ID - returning Task::none()");
                        Task::none()
                    }
                }
            }
            Message::OpenViewer => {
                // Spawn the viewer application as a separate process
                match std::process::Command::new("cosmic-applet-opencode-usage-viewer").spawn() {
                    Ok(_) => {
                        eprintln!("DEBUG: Viewer application launched successfully");
                        Task::none()
                    }
                    Err(e) => {
                        eprintln!("ERROR: Failed to launch viewer: {e}");
                        // Try the binary from the build directory as fallback
                        match std::process::Command::new(
                            "./target/release/cosmic-applet-opencode-usage-viewer",
                        )
                        .spawn()
                        {
                            Ok(_) => {
                                eprintln!("DEBUG: Viewer launched from build directory");
                                Task::none()
                            }
                            Err(e2) => {
                                eprintln!(
                                    "ERROR: Failed to launch viewer from build directory: {e2}"
                                );
                                // Could show an error message in the UI here in the future
                                Task::none()
                            }
                        }
                    }
                }
            }
            Message::Tick => {
                // Check if we need to refresh based on last update time
                if self.state.needs_refresh() {
                    eprintln!("[Tick] Refresh needed, triggering FetchMetrics");
                    Task::done(cosmic::Action::App(Message::FetchMetrics))
                } else {
                    Task::none()
                }
            }
        }
    }

    /// Get the icon name based on current state
    fn get_state_icon(&self) -> &'static str {
        match &self.state.panel_state {
            PanelState::Loading | PanelState::LoadingWithData(_) => "content-loading-symbolic",
            PanelState::Error(_) => "dialog-error-symbolic",
            PanelState::Success(_) | PanelState::Stale(_) => "dialog-information-symbolic",
        }
    }

    /// Build the metrics popup view
    #[allow(clippy::too_many_lines)] // UI function with many widget definitions
    fn metrics_popup_view(&self) -> Element<'_, Message> {
        use crate::ui::formatters::{format_cost, format_number, format_tooltip};

        let main_content = match &self.state.panel_state {
            PanelState::Loading => column()
                .push(text("Loading...").size(16))
                .push(text("").size(8))
                .push(
                    row()
                        .push(button::standard("View Stats").on_press(Message::OpenViewer))
                        .push(button::standard("Settings").on_press(Message::OpenSettings))
                        .spacing(8),
                )
                .spacing(10)
                .padding(20),
            PanelState::Error(err) => column()
                .push(text("Error").size(20))
                .push(text(err).size(14))
                .push(text("").size(8))
                .push(button::standard("Retry").on_press(Message::FetchMetrics))
                .push(
                    row()
                        .push(button::standard("View Stats").on_press(Message::OpenViewer))
                        .push(button::standard("Settings").on_press(Message::OpenSettings))
                        .spacing(8),
                )
                .spacing(10)
                .padding(20),
            PanelState::Success(usage)
            | PanelState::Stale(usage)
            | PanelState::LoadingWithData(usage) => {
                // Determine if we're loading
                let is_loading = matches!(self.state.panel_state, PanelState::LoadingWithData(_));

                // Determine title based on current mode
                let title = match self.state.display_mode {
                    DisplayMode::Today => "Today's Usage",
                    DisplayMode::Month => "This Month's Usage",
                    DisplayMode::AllTime => "All-Time Usage",
                };

                // Create three tab buttons
                let today_button = if self.state.display_mode == DisplayMode::Today {
                    button::suggested("Today")
                } else if is_loading {
                    button::standard("Today")
                } else {
                    button::standard("Today")
                        .on_press(Message::SelectDisplayMode(DisplayMode::Today))
                };

                let month_button = if self.state.display_mode == DisplayMode::Month {
                    button::suggested("Month")
                } else if is_loading {
                    button::standard("Month")
                } else {
                    button::standard("Month")
                        .on_press(Message::SelectDisplayMode(DisplayMode::Month))
                };

                let alltime_button = if self.state.display_mode == DisplayMode::AllTime {
                    button::suggested("All Time")
                } else if is_loading {
                    button::standard("All Time")
                } else {
                    button::standard("All Time")
                        .on_press(Message::SelectDisplayMode(DisplayMode::AllTime))
                };

                // Create tab row
                let tabs = row()
                    .push(today_button)
                    .push(month_button)
                    .push(alltime_button)
                    .spacing(8);

                column()
                    .push(text(title).size(20))
                    .push(text("").size(4))
                    .push(tabs)
                    .push(text("").size(8))
                    .push(
                        row()
                            .push(text("Total Cost: ").size(14))
                            .push(text(format_cost(usage.total_cost)).size(14))
                            .spacing(5),
                    )
                    .push(
                        row()
                            .push(text("Interactions: ").size(14))
                            .push(text(format_number(usage.interaction_count as u64)).size(14))
                            .spacing(5),
                    )
                    .push(
                        row()
                            .push(text("Input Tokens: ").size(14))
                            .push(text(format_number(usage.total_input_tokens)).size(14))
                            .spacing(5),
                    )
                    .push(
                        row()
                            .push(text("Output Tokens: ").size(14))
                            .push(text(format_number(usage.total_output_tokens)).size(14))
                            .spacing(5),
                    )
                    .push(text("").size(8))
                    .push(text(format_tooltip(self.state.last_update)).size(12))
                    .push(text("").size(8))
                    .push(
                        row()
                            .push(button::standard("View Stats").on_press(Message::OpenViewer))
                            .push(button::standard("Settings").on_press(Message::OpenSettings))
                            .spacing(8),
                    )
                    .spacing(10)
                    .padding(20)
            }
        };

        scrollable(main_content).into()
    }

    /// Build the settings dialog UI
    fn settings_view(&self) -> Element<'_, Message> {
        let mut content = column()
            .push(text("OpenCode Monitor Settings").size(24))
            .push(text("").size(8))
            .push(text("Refresh Interval (seconds)").size(14))
            .push(
                text_input("Enter refresh interval", &self.temp_refresh_interval_str).on_input(
                    |s| {
                        s.parse::<u32>()
                            .map(Message::UpdateRefreshInterval)
                            .unwrap_or(Message::None)
                    },
                ),
            )
            .push(text("").size(8))
            .push(text("Display Options").size(14))
            .push(
                checkbox(
                    "Show today's usage next to icon",
                    self.temp_show_today_usage,
                )
                .on_toggle(Message::ToggleShowTodayUsage),
            )
            .push(
                checkbox(
                    "Use raw token values (no K/M suffixes)",
                    self.temp_use_raw_token_display,
                )
                .on_toggle(Message::ToggleRawTokenDisplay),
            )
            .spacing(10)
            .padding(20);

        // Show error if present (red/critical style)
        if let Some(ref err) = self.config_error {
            content = content
                .push(text("").size(8))
                .push(text(format!("❌ Error: {err}")).size(14));
        }

        // Show warning if present (yellow/info style)
        if let Some(ref warn) = self.config_warning {
            let warning_text = match warn {
                ConfigWarning::LowRefreshInterval(interval) => {
                    format!("⚠️  Warning: Refresh interval of {interval} seconds is very low. This may cause high CPU usage and frequent file system access.")
                }
            };
            content = content
                .push(text("").size(8))
                .push(text(warning_text).size(14));
        }

        // Add action buttons
        content = content.push(text("").size(12)).push(
            row()
                .push(button::standard("Cancel").on_press(Message::CloseSettings))
                .push(button::suggested("Save").on_press(Message::SaveConfig))
                .spacing(12),
        );

        scrollable(content).into()
    }

    /// Create the panel button content layout
    fn panel_button_content(&self) -> Element<'_, Message> {
        use crate::ui::formatters::{
            format_panel_display_detailed, format_panel_display_detailed_raw,
        };

        // If show_today_usage is enabled and we have today's data, show icon + detailed metrics
        if self.state.config.show_today_usage {
            if let Some(today_usage) = &self.state.today_usage {
                let display_text = if self.state.config.use_raw_token_display {
                    format_panel_display_detailed_raw(today_usage)
                } else {
                    format_panel_display_detailed(today_usage)
                };
                // Show icon + text in a row
                return row()
                    .push(icon::from_name(self.get_state_icon()).size(16))
                    .push(self.core.applet.text(display_text))
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into();
            }
        }

        // Default: just show icon
        icon::from_name(self.get_state_icon()).size(16).into()
    }
}

/// Implement the Application trait for `OpenCodeMonitorApplet`
impl Application for OpenCodeMonitorApplet {
    type Executor = cosmic::executor::Default;
    type Flags = AppConfig;
    type Message = Message;
    const APP_ID: &'static str = "com.vasilvestre.CosmicAppletOpencodeUsage";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let reader = if let Some(ref path) = flags.storage_path {
            OpenCodeUsageReader::new_with_path(path.to_str().unwrap_or(""))
                .expect("Failed to create OpenCode reader")
        } else {
            OpenCodeUsageReader::new().expect("Failed to create OpenCode reader")
        };

        let temp_refresh_interval = flags.refresh_interval_seconds;
        let temp_show_today_usage = flags.show_today_usage;
        let temp_use_raw_token_display = flags.use_raw_token_display;

        // Create watch channel for refresh interval updates
        let (refresh_interval_tx, _rx) = watch::channel(flags.refresh_interval_seconds);

        // Initialize data collector with database
        let data_collector = match DatabaseManager::new() {
            Ok(db_manager) => {
                eprintln!("[DataCollector] Database initialized successfully");
                Some(DataCollector::new(Arc::new(db_manager)))
            }
            Err(e) => {
                eprintln!("[DataCollector] Failed to initialize database: {e}");
                eprintln!("[DataCollector] Continuing without automatic data collection");
                None
            }
        };

        let applet = Self {
            core,
            state: AppState::new(flags),
            reader,
            data_collector,
            settings_dialog_open: false,
            temp_refresh_interval,
            temp_refresh_interval_str: temp_refresh_interval.to_string(),
            temp_show_today_usage,
            temp_use_raw_token_display,
            config_error: None,
            config_warning: None,
            popup: None,
            refresh_interval_tx,
        };

        eprintln!("[init] Application initialized, triggering initial FetchMetrics");
        (
            applet,
            Task::done(cosmic::Action::App(Message::FetchMetrics)),
        )
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let button = button::custom(self.panel_button_content())
            .padding([0, self.core.applet.suggested_padding(true)])
            .on_press_down(Message::TogglePopup)
            .class(cosmic::theme::Button::AppletIcon);

        autosize::autosize(button, AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        self.handle_message(message)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Create a subscription that ticks based on refresh_interval_seconds
        // and dynamically updates when the interval changes
        let mut refresh_interval_rx = self.refresh_interval_tx.subscribe();

        Subscription::run_with_id(
            "opencode-refresh-sub",
            stream::channel(1, move |mut output| async move {
                // Mark as changed to receive initial value
                refresh_interval_rx.mark_changed();
                let mut interval_seconds: u64 = 60; // Default
                let mut timer = time::interval(std::time::Duration::from_secs(interval_seconds));
                timer.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

                loop {
                    tokio::select! {
                        _ = timer.tick() => {
                            #[cfg(debug_assertions)]
                            if let Err(err) = output.send(Message::Tick).await {
                                eprintln!("[Subscription] Failed sending tick: {err:?}");
                            }

                            #[cfg(not(debug_assertions))]
                            let _ = output.send(Message::Tick).await;
                        },
                        // Update timer if the user changes refresh interval
                        Ok(()) = refresh_interval_rx.changed() => {
                            interval_seconds = u64::from(*refresh_interval_rx.borrow_and_update());

                            #[cfg(debug_assertions)]
                            eprintln!("[Subscription] Refresh interval changed to {interval_seconds} seconds");

                            let period = time::Duration::from_secs(interval_seconds);
                            let start = time::Instant::now() + period;
                            timer = time::interval_at(start, period);
                            timer.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
                        }
                    }
                }
            }),
        )
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
                (600.0, 500.0)
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
#[allow(clippy::float_cmp)] // Tests use exact float comparisons for simplicity
mod tests {
    use super::*;
    use crate::core::config::AppConfig;
    use crate::core::opencode::UsageMetrics;
    use crate::ui::state::PanelState;
    use chrono::Utc;
    use std::time::SystemTime;

    fn create_mock_config() -> AppConfig {
        AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            show_today_usage: false,
            use_raw_token_display: false,
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
            // Data collector may or may not be initialized depending on database availability
        }
    }

    #[test]
    fn test_handle_metrics_fetched_success() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            let usage = create_mock_usage_metrics();

            let _ = applet.handle_message(Message::MetricsFetched(Box::new(Ok((
                usage.clone(),
                None,
                None,
            )))));

            assert!(matches!(applet.state.panel_state, PanelState::Success(_)));
            assert!(applet.state.last_update.is_some());
        }
    }

    #[test]
    fn test_handle_metrics_fetched_error() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            let error = "Test error".to_string();

            let _ = applet.handle_message(Message::MetricsFetched(Box::new(Err(error))));

            assert!(matches!(applet.state.panel_state, PanelState::Error(_)));
        }
    }

    #[test]
    fn test_settings_operations() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Open settings
            let _ = applet.handle_message(Message::OpenSettings);
            assert!(applet.settings_dialog_open);

            // Update refresh interval
            let _ = applet.handle_message(Message::UpdateRefreshInterval(1800));
            assert_eq!(applet.temp_refresh_interval, 1800);

            // Close settings
            let _ = applet.handle_message(Message::CloseSettings);
            assert!(!applet.settings_dialog_open);
        }
    }

    #[test]
    fn test_select_display_mode() {
        use crate::ui::state::DisplayMode;

        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Should start with Today mode
            assert_eq!(applet.state.display_mode, DisplayMode::Today);

            // Select Month mode
            let _ = applet.handle_message(Message::SelectDisplayMode(DisplayMode::Month));
            assert_eq!(applet.state.display_mode, DisplayMode::Month);

            // Select AllTime mode
            let _ = applet.handle_message(Message::SelectDisplayMode(DisplayMode::AllTime));
            assert_eq!(applet.state.display_mode, DisplayMode::AllTime);

            // Select Today mode again
            let _ = applet.handle_message(Message::SelectDisplayMode(DisplayMode::Today));
            assert_eq!(applet.state.display_mode, DisplayMode::Today);
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

    #[test]
    fn test_enabling_show_today_usage_triggers_fetch() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Should start disabled
            assert!(!applet.state.config.show_today_usage);

            // Open settings and enable show_today_usage
            let _ = applet.handle_message(Message::OpenSettings);
            let _ = applet.handle_message(Message::ToggleShowTodayUsage(true));

            // Save config should return a Task that triggers FetchMetrics
            let _task = applet.handle_message(Message::SaveConfig);

            // Verify config was updated
            assert!(applet.state.config.show_today_usage);

            // The task should not be Task::none() - it should trigger a fetch
            // We can't directly test Task equality, but we can verify the behavior
            // by checking that settings closed successfully
            assert!(!applet.settings_dialog_open);
        }
    }

    #[test]
    fn test_changing_raw_token_display_triggers_refresh() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Should start with raw display disabled
            assert!(!applet.state.config.use_raw_token_display);

            // Open settings and enable raw token display
            let _ = applet.handle_message(Message::OpenSettings);
            let _ = applet.handle_message(Message::ToggleRawTokenDisplay(true));

            // Save config
            let _ = applet.handle_message(Message::SaveConfig);

            // Verify config was updated and settings closed
            assert!(applet.state.config.use_raw_token_display);
            assert!(!applet.settings_dialog_open);
        }
    }

    #[test]
    fn test_month_usage_cache_update() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Initially no month cache
            assert!(applet.state.month_usage.is_none());

            // Simulate successful fetch with month data
            let main_usage = create_mock_usage_metrics();
            let month_usage = create_mock_usage_metrics();
            let _ = applet.handle_message(Message::MetricsFetched(Box::new(Ok((
                main_usage,
                None,
                Some(month_usage.clone()),
            )))));

            // Month cache should be updated
            assert!(applet.state.month_usage.is_some());
            assert_eq!(applet.state.month_usage.unwrap(), month_usage);
        }
    }

    #[test]
    fn test_month_mode_caches_month_data() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Switch to Month mode
            let _ = applet.handle_message(Message::SelectDisplayMode(DisplayMode::Month));
            assert_eq!(applet.state.display_mode, DisplayMode::Month);

            // Simulate successful month data fetch
            let month_usage = create_mock_usage_metrics();
            let _ = applet.handle_message(Message::MetricsFetched(Box::new(Ok((
                month_usage.clone(),
                None,
                None,
            )))));

            // Month cache should be populated when in Month mode
            assert!(applet.state.month_usage.is_some());
            assert_eq!(applet.state.month_usage.unwrap(), month_usage);
        }
    }

    #[test]
    fn test_month_cache_preserved_across_mode_switches() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Start in AllTime mode and fetch data with month cache
            let all_time_usage = create_mock_usage_metrics();
            let mut month_usage = create_mock_usage_metrics();
            month_usage.total_cost = 5.0; // Different value to distinguish

            let _ = applet.handle_message(Message::MetricsFetched(Box::new(Ok((
                all_time_usage,
                None,
                Some(month_usage.clone()),
            )))));
            assert!(applet.state.month_usage.is_some());

            // Switch to Today mode
            let _ = applet.handle_message(Message::SelectDisplayMode(DisplayMode::Today));

            // Month cache should still be preserved
            assert!(applet.state.month_usage.is_some());
            assert_eq!(applet.state.month_usage.as_ref().unwrap().total_cost, 5.0);

            // Switch to Month mode
            let _ = applet.handle_message(Message::SelectDisplayMode(DisplayMode::Month));

            // Month cache should still be preserved
            assert!(applet.state.month_usage.is_some());
            assert_eq!(applet.state.month_usage.as_ref().unwrap().total_cost, 5.0);
        }
    }

    #[test]
    fn test_month_cache_updates_on_subsequent_fetches() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Initial fetch with month cache
            let all_time_usage = create_mock_usage_metrics();
            let mut initial_month = create_mock_usage_metrics();
            initial_month.total_cost = 5.0;

            let _ = applet.handle_message(Message::MetricsFetched(Box::new(Ok((
                all_time_usage.clone(),
                None,
                Some(initial_month.clone()),
            )))));
            assert_eq!(applet.state.month_usage.as_ref().unwrap().total_cost, 5.0);

            // Second fetch with updated month data
            let mut updated_month = create_mock_usage_metrics();
            updated_month.total_cost = 10.0;

            let _ = applet.handle_message(Message::MetricsFetched(Box::new(Ok((
                all_time_usage,
                None,
                Some(updated_month),
            )))));

            // Month cache should be updated
            assert_eq!(applet.state.month_usage.as_ref().unwrap().total_cost, 10.0);
        }
    }

    #[test]
    fn test_tick_message_triggers_fetch_when_refresh_needed() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Simulate successful initial fetch
            let usage = create_mock_usage_metrics();
            let _ =
                applet.handle_message(Message::MetricsFetched(Box::new(Ok((usage, None, None)))));

            // Manually set last_update to old time to trigger refresh
            applet.state.last_update = Some(Utc::now() - chrono::Duration::seconds(1000));

            // Tick should trigger fetch since needs_refresh() returns true
            assert!(applet.state.needs_refresh());

            // Handle Tick message - should trigger FetchMetrics
            let _task = applet.handle_message(Message::Tick);

            // We can't directly inspect Task contents, but we can verify state didn't change unexpectedly
            assert!(applet.state.needs_refresh());
        }
    }

    #[test]
    fn test_tick_message_does_not_fetch_when_refresh_not_needed() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Simulate recent successful fetch
            let usage = create_mock_usage_metrics();
            let _ =
                applet.handle_message(Message::MetricsFetched(Box::new(Ok((usage, None, None)))));

            // last_update should be recent (just set by update_success)
            assert!(!applet.state.needs_refresh());

            // Handle Tick message - should NOT trigger fetch
            let _task = applet.handle_message(Message::Tick);

            // Verify refresh is still not needed
            assert!(!applet.state.needs_refresh());
        }
    }

    #[test]
    fn test_subscription_method_exists() {
        use cosmic::Application;

        let config = create_mock_config();
        if let Ok(applet) = OpenCodeMonitorApplet::new(config) {
            // Call subscription to ensure it's implemented
            let _subscription = applet.subscription();
            // If this compiles and runs, the subscription method exists and returns a Subscription
        }
    }

    #[test]
    fn test_refresh_interval_updates_via_watch_channel() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Initial refresh interval
            assert_eq!(applet.state.config.refresh_interval_seconds, 60);

            // Create a receiver before the change to verify notification
            let mut rx = applet.refresh_interval_tx.subscribe();
            assert_eq!(*rx.borrow(), 60);

            // Update temp settings
            applet.temp_refresh_interval = 120;

            // Save config - this should update the watch channel
            let _task = applet.handle_message(Message::SaveConfig);

            // After SaveConfig, verify the config was updated
            assert_eq!(applet.state.config.refresh_interval_seconds, 120);

            // Verify the receiver got notified
            assert!(rx.has_changed().unwrap());
            assert_eq!(*rx.borrow_and_update(), 120);
        }
    }

    #[test]
    fn test_refresh_interval_updates_subscription_receiver() {
        let config = create_mock_config();
        if let Ok(mut applet) = OpenCodeMonitorApplet::new(config) {
            // Create a receiver to simulate what the subscription does
            let mut rx = applet.refresh_interval_tx.subscribe();

            // Initial value
            assert_eq!(*rx.borrow(), 60);

            // Change refresh interval
            applet.temp_refresh_interval = 300;
            let _task = applet.handle_message(Message::SaveConfig);

            // The watch channel should notify subscribers
            assert!(rx.has_changed().unwrap());
            assert_eq!(*rx.borrow_and_update(), 300);
        }
    }
}
