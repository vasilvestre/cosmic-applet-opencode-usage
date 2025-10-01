// SPDX-License-Identifier: GPL-3.0-only

//! UI message types for the applet

use crate::core::opencode::UsageMetrics;

/// Messages that can be sent to update the application state
#[derive(Debug, Clone)]
pub enum Message {
    /// Trigger a metrics fetch from OpenCode usage files
    FetchMetrics,
    /// Metrics fetch completed (success or error)
    MetricsFetched(Result<UsageMetrics, String>),
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
    /// Toggle show_today_usage setting
    ToggleShowTodayUsage(bool),
    /// Save configuration
    SaveConfig,
    /// Toggle popup visibility
    TogglePopup,
    /// Toggle display mode between Today and AllTime
    ToggleDisplayMode,
    /// No-op message for event handling
    None,
}
