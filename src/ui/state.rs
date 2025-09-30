// SPDX-License-Identifier: GPL-3.0-only

//! Panel state management for the UI

use crate::core::models::CopilotUsage;

/// Represents the current state of the panel display
#[derive(Debug, Clone)]
pub enum PanelState {
    /// Initial state when loading data
    Loading,
    /// Data successfully loaded and fresh
    Success(CopilotUsage),
    /// Data loaded but potentially outdated
    Stale(CopilotUsage),
    /// Error occurred during data loading
    Error(String),
}

impl PanelState {
    /// Returns true if the state is Loading
    pub fn is_loading(&self) -> bool {
        matches!(self, PanelState::Loading)
    }

    /// Returns true if the state is Error
    pub fn is_error(&self) -> bool {
        matches!(self, PanelState::Error(_))
    }

    /// Returns true if the state has data (Success or Stale)
    pub fn has_data(&self) -> bool {
        matches!(self, PanelState::Success(_) | PanelState::Stale(_))
    }

    /// Returns a reference to the usage data if available (Success or Stale)
    pub fn get_usage(&self) -> Option<&CopilotUsage> {
        match self {
            PanelState::Success(usage) | PanelState::Stale(usage) => Some(usage),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::UsageBreakdown;

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
    fn test_panel_state_variants_exist() {
        let _loading = PanelState::Loading;
        let _error = PanelState::Error("test error".to_string());
        let usage = create_mock_copilot_usage();
        let _success = PanelState::Success(usage.clone());
        let _stale = PanelState::Stale(usage);
    }

    #[test]
    fn test_panel_state_error_message() {
        let error = PanelState::Error("API failed".to_string());
        match error {
            PanelState::Error(msg) => assert_eq!(msg, "API failed"),
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_panel_state_success_holds_data() {
        let usage = create_mock_copilot_usage();
        let success = PanelState::Success(usage.clone());
        match success {
            PanelState::Success(data) => assert_eq!(data, usage),
            _ => panic!("Expected Success variant"),
        }
    }

    #[test]
    fn test_is_loading_returns_true_for_loading_state() {
        let state = PanelState::Loading;
        assert!(state.is_loading());
        
        let state = PanelState::Error("test".to_string());
        assert!(!state.is_loading());
    }

    #[test]
    fn test_is_error_returns_true_for_error_state() {
        let state = PanelState::Error("test".to_string());
        assert!(state.is_error());
        
        let state = PanelState::Loading;
        assert!(!state.is_error());
    }

    #[test]
    fn test_has_data_returns_true_for_success_and_stale() {
        let usage = create_mock_copilot_usage();
        
        let success = PanelState::Success(usage.clone());
        assert!(success.has_data());
        
        let stale = PanelState::Stale(usage);
        assert!(stale.has_data());
        
        let loading = PanelState::Loading;
        assert!(!loading.has_data());
        
        let error = PanelState::Error("test".to_string());
        assert!(!error.has_data());
    }

    #[test]
    fn test_get_usage_returns_data_for_success_and_stale() {
        let usage = create_mock_copilot_usage();
        
        let success = PanelState::Success(usage.clone());
        assert_eq!(success.get_usage(), Some(&usage));
        
        let stale = PanelState::Stale(usage.clone());
        assert_eq!(stale.get_usage(), Some(&usage));
        
        let loading = PanelState::Loading;
        assert_eq!(loading.get_usage(), None);
        
        let error = PanelState::Error("test".to_string());
        assert_eq!(error.get_usage(), None);
    }
}
