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

/// Generates detailed text for the popup display
///
/// Returns a vector of strings, each representing a line in the popup:
/// - Loading: Single line with loading message
/// - Error: Two lines with error header and message
/// - Success/Stale: Four lines with detailed metrics breakdown including acceptance rate
///
/// # Arguments
/// * `state` - The current panel state
///
/// # Returns
/// A vector of strings suitable for multi-line popup display
///
/// # Examples
/// ```
/// use cosmic_applet_copilot_quota_tracker::ui::{view_popup_text, PanelState};
/// use cosmic_applet_copilot_quota_tracker::core::models::CopilotUsage;
///
/// let loading = PanelState::Loading;
/// assert_eq!(view_popup_text(&loading), vec!["Loading metrics..."]);
/// ```
pub fn view_popup_text(state: &PanelState) -> Vec<String> {
    match state {
        PanelState::Loading => vec!["Loading metrics...".to_string()],
        PanelState::Error(err) => vec![
            "Error loading metrics".to_string(),
            err.clone(),
        ],
        PanelState::Success(usage) | PanelState::Stale(usage) => {
            let acceptances = usage.total_acceptances_count;
            let suggestions = usage.total_suggestions_count;
            let lines_accepted = usage.total_lines_accepted;
            let lines_suggested = usage.total_lines_suggested;
            
            // Calculate acceptance rate percentage
            let acceptance_rate = if suggestions > 0 {
                (acceptances as f64 / suggestions as f64) * 100.0
            } else {
                0.0
            };
            
            vec![
                format!("Acceptances: {}", format_number(acceptances as u64)),
                format!("Suggestions: {} ({:.1}%)", format_number(suggestions as u64), acceptance_rate),
                format!("Lines Accepted: {}", format_number(lines_accepted as u64)),
                format!("Lines Suggested: {}", format_number(lines_suggested as u64)),
            ]
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
    /// Open settings dialog
    OpenSettings,
    /// Close settings dialog
    CloseSettings,
    /// Update organization name field
    UpdateOrgName(String),
    /// Update Personal Access Token field
    UpdatePat(String),
    /// Update refresh interval field
    UpdateRefreshInterval(u32),
    /// Save configuration to disk and keyring
    SaveConfig,
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

    // Tests for view_popup_text()
    #[test]
    fn test_view_popup_text_loading() {
        let state = PanelState::Loading;
        let result = view_popup_text(&state);
        assert_eq!(result, vec!["Loading metrics..."]);
    }

    #[test]
    fn test_view_popup_text_error() {
        let error_msg = "Network timeout".to_string();
        let state = PanelState::Error(error_msg.clone());
        let result = view_popup_text(&state);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "Error loading metrics");
        assert_eq!(result[1], error_msg);
    }

    #[test]
    fn test_view_popup_text_success() {
        let usage = create_mock_copilot_usage(); // 50/100 acceptances, 75/200 lines
        let state = PanelState::Success(usage);
        let result = view_popup_text(&state);
        
        // Expected format:
        // "Acceptances: 50"
        // "Suggestions: 100 (50.0%)"
        // "Lines Accepted: 75"
        // "Lines Suggested: 200"
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "Acceptances: 50");
        assert_eq!(result[1], "Suggestions: 100 (50.0%)");
        assert_eq!(result[2], "Lines Accepted: 75");
        assert_eq!(result[3], "Lines Suggested: 200");
    }
}
