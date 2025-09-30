// SPDX-License-Identifier: GPL-3.0-only

//! Panel state management for the UI

use crate::core::config::AppConfig;
use crate::core::models::CopilotUsage;
use chrono::{DateTime, Utc};

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

/// Application state holding panel state and metadata
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current panel state
    pub panel_state: PanelState,
    /// Timestamp of last successful data update
    pub last_update: Option<DateTime<Utc>>,
    /// Application configuration
    pub config: AppConfig,
}

impl AppState {
    /// Creates a new AppState with Loading state
    pub fn new(config: AppConfig) -> Self {
        AppState {
            panel_state: PanelState::Loading,
            last_update: None,
            config,
        }
    }

    /// Updates state with successful data fetch
    pub fn update_success(&mut self, usage: CopilotUsage) {
        self.panel_state = PanelState::Success(usage);
        self.last_update = Some(Utc::now());
    }

    /// Updates state with error
    pub fn update_error(&mut self, error: String) {
        self.panel_state = PanelState::Error(error);
        // Don't update last_update timestamp on error
    }

    /// Marks current data as stale
    pub fn mark_stale(&mut self) {
        if let Some(usage) = self.panel_state.get_usage() {
            self.panel_state = PanelState::Stale(usage.clone());
        }
    }

    /// Checks if data should be refreshed based on last update time and config interval
    pub fn needs_refresh(&self) -> bool {
        match self.last_update {
            None => true,
            Some(last) => {
                let elapsed = Utc::now() - last;
                elapsed > chrono::Duration::seconds(self.config.refresh_interval_seconds as i64)
            }
        }
    }

    /// Checks if the application configuration is valid
    pub fn is_initialized(&self) -> bool {
        self.config.validate().is_ok()
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

    fn create_mock_config() -> AppConfig {
        AppConfig {
            organization_name: "test-org".to_string(),
            refresh_interval_seconds: 900,
        }
    }

    // AppState tests
    #[test]
    fn test_app_state_initial_state() {
        let config = create_mock_config();
        let state = AppState::new(config.clone());
        
        assert!(state.panel_state.is_loading());
        assert_eq!(state.last_update, None);
        assert_eq!(state.config, config);
    }

    #[test]
    fn test_app_state_update_to_success() {
        let config = create_mock_config();
        let mut state = AppState::new(config);
        let usage = create_mock_copilot_usage();
        
        state.update_success(usage.clone());
        
        assert!(matches!(state.panel_state, PanelState::Success(_)));
        assert_eq!(state.panel_state.get_usage(), Some(&usage));
        assert!(state.last_update.is_some());
    }

    #[test]
    fn test_app_state_update_to_error() {
        let config = create_mock_config();
        let mut state = AppState::new(config);
        
        state.update_error("API failed".to_string());
        
        assert!(state.panel_state.is_error());
        match &state.panel_state {
            PanelState::Error(msg) => assert_eq!(msg, "API failed"),
            _ => panic!("Expected Error state"),
        }
        // last_update should remain None (no successful update yet)
        assert_eq!(state.last_update, None);
    }

    #[test]
    fn test_app_state_mark_stale() {
        let config = create_mock_config();
        let mut state = AppState::new(config);
        let usage = create_mock_copilot_usage();
        
        // First update to success
        state.update_success(usage.clone());
        assert!(matches!(state.panel_state, PanelState::Success(_)));
        
        // Then mark as stale
        state.mark_stale();
        assert!(matches!(state.panel_state, PanelState::Stale(_)));
        assert_eq!(state.panel_state.get_usage(), Some(&usage));
    }

    #[test]
    fn test_needs_refresh_no_update() {
        let config = create_mock_config();
        let state = AppState::new(config);
        
        // No last_update means needs refresh
        assert!(state.needs_refresh());
    }

    #[test]
    fn test_needs_refresh_recent_update() {
        let config = create_mock_config(); // 900 seconds (15 min) interval
        let mut state = AppState::new(config);
        let usage = create_mock_copilot_usage();
        
        state.update_success(usage);
        
        // Just updated, should not need refresh
        assert!(!state.needs_refresh());
    }

    #[test]
    fn test_needs_refresh_stale_data() {
        let config = create_mock_config();
        let mut state = AppState::new(config);
        
        // Manually set old timestamp (16 minutes ago, beyond 15 min interval)
        state.last_update = Some(Utc::now() - chrono::Duration::seconds(960));
        
        assert!(state.needs_refresh());
    }

    #[test]
    fn test_is_initialized_valid_config() {
        let config = create_mock_config();
        let state = AppState::new(config);
        
        assert!(state.is_initialized());
    }

    #[test]
    fn test_is_initialized_invalid_config() {
        let invalid_config = AppConfig {
            organization_name: "".to_string(), // Invalid: empty org name
            refresh_interval_seconds: 900,
        };
        let state = AppState::new(invalid_config);
        
        assert!(!state.is_initialized());
    }

    // PanelState tests
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
