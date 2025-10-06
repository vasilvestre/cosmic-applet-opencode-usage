// SPDX-License-Identifier: GPL-3.0-only

//! UI message types for the applet

use crate::core::config::AppConfig;
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
    /// Contains generation ID (to ignore outdated responses), main metrics,
    /// optionally today's metrics, and optionally month metrics for panel display
    MetricsFetched(u64, Box<MetricsFetchResult>),
    /// Config changed externally (from another instance via COSMIC's `watch_config`)
    ConfigChanged(AppConfig),
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
    /// Toggle a panel metric on/off (add or remove from the list)
    TogglePanelMetric(crate::core::config::PanelMetric),
    /// Reset panel metrics to default (all 5 metrics)
    ResetPanelMetricsToDefaults,
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
    /// Open the viewer application
    OpenViewer,
    /// No-op message for event handling
    None,
}
