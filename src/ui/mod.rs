// SPDX-License-Identifier: GPL-3.0-only

//! UI utilities for the COSMIC applet

pub mod formatters;
pub mod state;

pub use state::PanelState;

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
}
