// SPDX-License-Identifier: GPL-3.0-only

//! UI utilities for the COSMIC applet

pub mod formatters;
pub mod state;

pub use state::PanelState;

use formatters::{format_number, get_primary_metric};

/// Generates the text displayed in the applet panel
///
/// Returns a formatted string based on the current state:
/// - Loading: Shows a loading indicator emoji
/// - Error: Shows an error indicator emoji  
/// - Success/Stale: Shows the formatted primary metric (acceptances count)
///
/// # Arguments
/// * `state` - The current panel state
///
/// # Returns
/// A string suitable for display in the panel
///
/// # Examples
/// ```
/// use cosmic_applet_copilot_quota_tracker::ui::{view_panel_text, PanelState};
/// use cosmic_applet_copilot_quota_tracker::core::models::CopilotUsage;
///
/// let loading = PanelState::Loading;
/// assert_eq!(view_panel_text(&loading), "⏳");
///
/// let error = PanelState::Error("Network error".to_string());
/// assert_eq!(view_panel_text(&error), "❌");
/// ```
pub fn view_panel_text(state: &PanelState) -> String {
    match state {
        PanelState::Loading => "⏳".to_string(),
        PanelState::Error(_) => "❌".to_string(),
        PanelState::Success(usage) | PanelState::Stale(usage) => {
            format_number(get_primary_metric(usage))
        }
    }
}

/// Messages for UI event handling
#[derive(Debug, Clone)]
pub enum Message {
    /// Trigger metrics fetch from GitHub API
    FetchMetrics,
    /// Theme changed event
    ThemeChanged,
    /// Update tooltip display
    UpdateTooltip,
    /// Metrics fetch completed with result (error as String for Clone compatibility)
    MetricsFetched(Result<crate::core::models::CopilotUsage, String>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{CopilotUsage, UsageBreakdown};

    fn create_mock_copilot_usage() -> CopilotUsage {
        CopilotUsage {
            total_suggestions_count: 100,
            total_acceptances_count: 50,
            total_lines_suggested: 200,
            total_lines_accepted: 75,
            day: "2025-09-30".to_string(),
            breakdown: vec![UsageBreakdown {
                language: "rust".to_string(),
                editor: "vscode".to_string(),
                suggestions_count: 100,
                acceptances_count: 50,
                lines_suggested: 200,
                lines_accepted: 75,
            }],
        }
    }

    #[test]
    fn test_message_variants_exist() {
        let _fetch = Message::FetchMetrics;
        let _theme = Message::ThemeChanged;
        let _tooltip = Message::UpdateTooltip;
    }

    #[test]
    fn test_message_metrics_fetched_success() {
        let usage = create_mock_copilot_usage();
        let msg = Message::MetricsFetched(Ok(usage.clone()));
        
        match msg {
            Message::MetricsFetched(Ok(data)) => assert_eq!(data, usage),
            _ => panic!("Expected MetricsFetched Ok variant"),
        }
    }

    #[test]
    fn test_message_metrics_fetched_error() {
        let error = "Network timeout occurred".to_string();
        let msg = Message::MetricsFetched(Err(error));
        
        assert!(matches!(msg, Message::MetricsFetched(Err(_))));
    }

    // Tests for view_panel_text()
    #[test]
    fn test_view_panel_text_loading() {
        let state = PanelState::Loading;
        assert_eq!(view_panel_text(&state), "⏳");
    }

    #[test]
    fn test_view_panel_text_error() {
        let state = PanelState::Error("Network error".to_string());
        assert_eq!(view_panel_text(&state), "❌");
    }

    #[test]
    fn test_view_panel_text_success() {
        let usage = create_mock_copilot_usage(); // Has 50 acceptances
        let state = PanelState::Success(usage);
        assert_eq!(view_panel_text(&state), "50"); // Should format the number
    }

    #[test]
    fn test_view_panel_text_stale() {
        let usage = create_mock_copilot_usage(); // Has 50 acceptances
        let state = PanelState::Stale(usage);
        assert_eq!(view_panel_text(&state), "50"); // Stale data still shows the number
    }
}
