// SPDX-License-Identifier: GPL-3.0-only

//! Panel state management for the UI

use crate::core::config::AppConfig;
use crate::core::opencode::UsageMetrics;
use chrono::{DateTime, Utc};

/// Represents the current state of the panel display
#[derive(Debug, Clone)]
pub enum PanelState {
    /// Initial state when loading data
    Loading,
    /// Data successfully loaded and fresh
    Success(UsageMetrics),
    /// Data loaded but potentially outdated
    Stale(UsageMetrics),
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
    pub fn get_usage(&self) -> Option<&UsageMetrics> {
        match self {
            PanelState::Success(usage) | PanelState::Stale(usage) => Some(usage),
            _ => None,
        }
    }
}

/// Display mode for usage metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Show all-time usage data
    AllTime,
    /// Show today's usage data only
    Today,
}

impl DisplayMode {
    /// Toggle between Today and AllTime modes
    pub fn toggle(&self) -> Self {
        match self {
            DisplayMode::AllTime => DisplayMode::Today,
            DisplayMode::Today => DisplayMode::AllTime,
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
    /// Current display mode (Today or AllTime)
    pub display_mode: DisplayMode,
    /// Today's usage for panel display (cached)
    pub today_usage: Option<UsageMetrics>,
}

impl AppState {
    /// Creates a new AppState with Loading state
    pub fn new(config: AppConfig) -> Self {
        AppState {
            panel_state: PanelState::Loading,
            last_update: None,
            config,
            display_mode: DisplayMode::Today,
            today_usage: None,
        }
    }

    /// Updates state with successful data fetch
    pub fn update_success(&mut self, usage: UsageMetrics) {
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

    /// Toggle the display mode between Today and AllTime
    pub fn toggle_display_mode(&mut self) {
        self.display_mode = self.display_mode.toggle();
    }

    /// Update today's usage for panel display
    pub fn update_today_usage(&mut self, usage: UsageMetrics) {
        self.today_usage = Some(usage);
    }

    /// Clear today's usage cache
    pub fn clear_today_usage(&mut self) {
        self.today_usage = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_mock_usage_metrics() -> UsageMetrics {
        UsageMetrics {
            total_input_tokens: 1000,
            total_output_tokens: 500,
            total_reasoning_tokens: 200,
            total_cache_write_tokens: 100,
            total_cache_read_tokens: 50,
            total_cost: 0.15,
            interaction_count: 5,
            timestamp: SystemTime::now(),
        }
    }

    fn create_mock_config() -> AppConfig {
        AppConfig {
            storage_path: None, // Use default OpenCode storage path
            refresh_interval_seconds: 900,
            show_today_usage: false,
            use_raw_token_display: false,
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
        let usage = create_mock_usage_metrics();
        
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
        let usage = create_mock_usage_metrics();
        
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
        let usage = create_mock_usage_metrics();
        
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
            storage_path: None,
            refresh_interval_seconds: 0, // Invalid: below minimum of 1
            show_today_usage: false,
            use_raw_token_display: false,
        };
        let state = AppState::new(invalid_config);
        
        assert!(!state.is_initialized());
    }

    #[test]
    fn test_display_mode_default() {
        let config = create_mock_config();
        let state = AppState::new(config);
        assert_eq!(state.display_mode, DisplayMode::Today);
    }

    #[test]
    fn test_display_mode_toggle() {
        let config = create_mock_config();
        let mut state = AppState::new(config);
        
        assert_eq!(state.display_mode, DisplayMode::Today);
        state.toggle_display_mode();
        assert_eq!(state.display_mode, DisplayMode::AllTime);
        state.toggle_display_mode();
        assert_eq!(state.display_mode, DisplayMode::Today);
    }

    #[test]
    fn test_display_mode_enum_toggle() {
        assert_eq!(DisplayMode::AllTime.toggle(), DisplayMode::Today);
        assert_eq!(DisplayMode::Today.toggle(), DisplayMode::AllTime);
    }

    #[test]
    fn test_update_today_usage() {
        let config = create_mock_config();
        let mut state = AppState::new(config);
        let usage = create_mock_usage_metrics();
        
        assert!(state.today_usage.is_none());
        
        state.update_today_usage(usage.clone());
        assert!(state.today_usage.is_some());
        assert_eq!(state.today_usage.unwrap(), usage);
    }

    #[test]
    fn test_clear_today_usage() {
        let config = create_mock_config();
        let mut state = AppState::new(config);
        let usage = create_mock_usage_metrics();
        
        state.update_today_usage(usage);
        assert!(state.today_usage.is_some());
        
        state.clear_today_usage();
        assert!(state.today_usage.is_none());
    }

    // PanelState tests
    #[test]
    fn test_panel_state_variants_exist() {
        let _loading = PanelState::Loading;
        let _error = PanelState::Error("test error".to_string());
        let usage = create_mock_usage_metrics();
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
        let usage = create_mock_usage_metrics();
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
        let usage = create_mock_usage_metrics();
        
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
        let usage = create_mock_usage_metrics();
        
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
