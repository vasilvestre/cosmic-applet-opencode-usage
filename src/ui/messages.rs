// SPDX-License-Identifier: GPL-3.0-only

//! UI message types for the applet

use crate::core::opencode::UsageMetrics;
use crate::ui::state::DisplayMode;

/// Result type for metrics fetch containing main, today, and month metrics
pub type MetricsFetchResult =
    Result<(UsageMetrics, Option<UsageMetrics>, Option<UsageMetrics>), String>;

/// Messages that can be sent to update the application state
#[derive(Debug, Clone)]
pub enum Message {
    /// Trigger a metrics fetch from `OpenCode` usage files
    FetchMetrics,
    /// Metrics fetch completed (success or error)
    /// Contains main metrics, optionally today's metrics, and optionally month metrics for panel display
    MetricsFetched(Box<MetricsFetchResult>),
    /// Theme changed (visual refresh needed)
    ThemeChanged,
    /// Tooltip needs update
    UpdateTooltip,
    /// Open settings dialog
    OpenSettings,
    /// Close settings dialog
    CloseSettings,
    /// Update refresh interval in settings
    UpdateRefreshInterval(u32),
    /// Toggle `show_today_usage` setting
    ToggleShowTodayUsage(bool),
    /// Toggle raw token display setting
    ToggleRawTokenDisplay(bool),
    /// Save configuration
    SaveConfig,
    /// Toggle popup visibility
    TogglePopup,
    /// Select a specific display mode (Today, Month, or `AllTime`)
    SelectDisplayMode(DisplayMode),
    /// Periodic timer tick for auto-refresh
    Tick,
    /// No-op message for event handling
    None,
}
