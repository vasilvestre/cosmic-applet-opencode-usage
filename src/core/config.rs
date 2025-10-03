// SPDX-License-Identifier: GPL-3.0-only

//! Configuration management for the `OpenCode` usage applet

use crate::ui::state::DisplayMode;
use cosmic_config::CosmicConfigEntry;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Application identifier for COSMIC config system
pub const APP_ID: &str = "com.vasilvestre.CosmicAppletOpencodeUsage";
pub const CONFIG_VERSION: u64 = 1;

/// Configuration error types
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ConfigError {
    #[error("Refresh interval must be between 1 and 3600 seconds (got {0})")]
    InvalidRefreshInterval(u32),
    #[error("Failed to load config: {0}")]
    LoadError(String),
    #[error("Failed to save config: {0}")]
    SaveError(String),
}

/// Configuration warning types (non-blocking)
#[derive(Debug, PartialEq, Clone)]
pub enum ConfigWarning {
    /// Refresh interval is very low (< 60 seconds), may cause high CPU usage
    LowRefreshInterval(u32),
}

/// Metric to display next to the icon in the panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelMetric {
    /// Show total cost (e.g., "$1.23")
    Cost,
    /// Show interaction count (e.g., "5x")
    Interactions,
    /// Show input tokens (e.g., "10k")
    InputTokens,
    /// Show output tokens (e.g., "5k")
    OutputTokens,
    /// Show reasoning/thinking tokens (e.g., "2k")
    ReasoningTokens,
}

impl Default for PanelMetric {
    fn default() -> Self {
        Self::Cost // Default to showing cost (most commonly requested metric)
    }
}

/// Application configuration
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    cosmic_config::cosmic_config_derive::CosmicConfigEntry,
)]
#[version = 1]
pub struct AppConfig {
    /// Path to `OpenCode` storage directory (optional, defaults to ~/.local/share/opencode/storage/part)
    pub storage_path: Option<PathBuf>,
    /// Refresh interval in seconds (default: 60 = 1 minute)
    pub refresh_interval_seconds: u32,
    /// Which metrics to show next to the icon in the panel (default: all metrics enabled)
    pub panel_metrics: Vec<PanelMetric>,
    /// Use raw token values instead of formatted (K/M) suffixes (default: false)
    pub use_raw_token_display: bool,
    /// Display mode for usage metrics (default: Today)
    pub display_mode: DisplayMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            storage_path: None, // Will use default path from OpenCodeUsageReader
            refresh_interval_seconds: 60,
            panel_metrics: vec![
                PanelMetric::Cost,
                PanelMetric::Interactions,
                PanelMetric::InputTokens,
                PanelMetric::OutputTokens,
                PanelMetric::ReasoningTokens,
            ],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        }
    }
}

impl AppConfig {
    /// Creates a new config with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from COSMIC config system
    /// Falls back to defaults if config doesn't exist or can't be loaded
    ///
    /// # Errors
    /// Returns an error if the config system cannot be accessed or initialized.
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_with_id(APP_ID)
    }

    /// Loads configuration with a custom app ID (useful for testing)
    #[cfg(test)]
    pub fn load_with_id(app_id: &str) -> Result<Self, ConfigError> {
        use cosmic::cosmic_config::{Config, ConfigGet};

        // Try to open config, if it fails, return defaults
        let config = Config::new(app_id, CONFIG_VERSION)
            .map_err(|e| ConfigError::LoadError(format!("Failed to open config: {e}")))?;

        // Load each field individually, using defaults for missing values
        let default = Self::default();

        Ok(Self {
            storage_path: config.get("storage_path").unwrap_or(default.storage_path),
            refresh_interval_seconds: config
                .get("refresh_interval_seconds")
                .unwrap_or(default.refresh_interval_seconds),
            panel_metrics: config.get("panel_metrics").unwrap_or(default.panel_metrics),
            use_raw_token_display: config
                .get("use_raw_token_display")
                .unwrap_or(default.use_raw_token_display),
            display_mode: config.get("display_mode").unwrap_or(default.display_mode),
        })
    }

    #[cfg(not(test))]
    fn load_with_id(app_id: &str) -> Result<Self, ConfigError> {
        use cosmic::cosmic_config::{Config, ConfigGet};

        // Try to open config, if it fails, return defaults
        let config = Config::new(app_id, CONFIG_VERSION)
            .map_err(|e| ConfigError::LoadError(format!("Failed to open config: {e}")))?;

        // Load each field individually, using defaults for missing values
        let default = Self::default();

        Ok(Self {
            storage_path: config.get("storage_path").unwrap_or(default.storage_path),
            refresh_interval_seconds: config
                .get("refresh_interval_seconds")
                .unwrap_or(default.refresh_interval_seconds),
            panel_metrics: config.get("panel_metrics").unwrap_or(default.panel_metrics),
            use_raw_token_display: config
                .get("use_raw_token_display")
                .unwrap_or(default.use_raw_token_display),
            display_mode: config.get("display_mode").unwrap_or(default.display_mode),
        })
    }

    /// Saves configuration to COSMIC config system
    ///
    /// # Errors
    /// Returns an error if the config cannot be saved to the COSMIC config system.
    pub fn save(&self) -> Result<(), ConfigError> {
        Self::save_with_id(self, APP_ID)
    }

    /// Saves configuration with a custom app ID (useful for testing)
    #[cfg(test)]
    pub fn save_with_id(&self, app_id: &str) -> Result<(), ConfigError> {
        use cosmic::cosmic_config::{Config, ConfigSet};

        let config = Config::new(app_id, CONFIG_VERSION)
            .map_err(|e| ConfigError::SaveError(format!("Failed to open config: {e}")))?;

        // Save each field individually
        config
            .set("storage_path", &self.storage_path)
            .map_err(|e| ConfigError::SaveError(format!("Failed to save storage_path: {e}")))?;
        config
            .set("refresh_interval_seconds", self.refresh_interval_seconds)
            .map_err(|e| {
                ConfigError::SaveError(format!("Failed to save refresh_interval_seconds: {e}"))
            })?;
        config
            .set("panel_metrics", &self.panel_metrics)
            .map_err(|e| ConfigError::SaveError(format!("Failed to save panel_metrics: {e}")))?;
        config
            .set("use_raw_token_display", self.use_raw_token_display)
            .map_err(|e| {
                ConfigError::SaveError(format!("Failed to save use_raw_token_display: {e}"))
            })?;
        config
            .set("display_mode", self.display_mode)
            .map_err(|e| ConfigError::SaveError(format!("Failed to save display_mode: {e}")))?;

        Ok(())
    }

    #[cfg(not(test))]
    fn save_with_id(&self, app_id: &str) -> Result<(), ConfigError> {
        use cosmic::cosmic_config::{Config, ConfigSet};

        let config = Config::new(app_id, CONFIG_VERSION)
            .map_err(|e| ConfigError::SaveError(format!("Failed to open config: {e}")))?;

        // Save each field individually
        config
            .set("storage_path", &self.storage_path)
            .map_err(|e| ConfigError::SaveError(format!("Failed to save storage_path: {e}")))?;
        config
            .set("refresh_interval_seconds", self.refresh_interval_seconds)
            .map_err(|e| {
                ConfigError::SaveError(format!("Failed to save refresh_interval_seconds: {e}"))
            })?;
        config
            .set("panel_metrics", &self.panel_metrics)
            .map_err(|e| ConfigError::SaveError(format!("Failed to save panel_metrics: {e}")))?;
        config
            .set("use_raw_token_display", self.use_raw_token_display)
            .map_err(|e| {
                ConfigError::SaveError(format!("Failed to save use_raw_token_display: {e}"))
            })?;
        config
            .set("display_mode", self.display_mode)
            .map_err(|e| ConfigError::SaveError(format!("Failed to save display_mode: {e}")))?;

        Ok(())
    }

    /// Validates the configuration, returning any warnings
    ///
    /// # Errors
    /// Returns an error if the configuration has invalid values (e.g., refresh interval out of range).
    pub fn validate(&self) -> Result<Option<ConfigWarning>, ConfigError> {
        validate_refresh_interval(self.refresh_interval_seconds)
    }
}

/// Validates refresh interval is within acceptable range (1-3600 seconds)
/// Returns a warning (not an error) if interval is < 60 seconds
///
/// # Errors
/// Returns an error if the interval is not within 1-3600 seconds.
pub fn validate_refresh_interval(interval: u32) -> Result<Option<ConfigWarning>, ConfigError> {
    if !(1..=3600).contains(&interval) {
        return Err(ConfigError::InvalidRefreshInterval(interval));
    }

    // Warn if interval is very low (< 60 seconds)
    if interval < 60 {
        return Ok(Some(ConfigWarning::LowRefreshInterval(interval)));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== PANEL METRIC TESTS (TDD - RED PHASE) =====

    #[test]
    fn test_panel_metric_default() {
        let metric = PanelMetric::default();
        assert_eq!(metric, PanelMetric::Cost);
    }

    #[test]
    fn test_panel_metric_variants() {
        // Test all variants exist and are distinct
        let cost = PanelMetric::Cost;
        let interactions = PanelMetric::Interactions;
        let input = PanelMetric::InputTokens;
        let output = PanelMetric::OutputTokens;
        let reasoning = PanelMetric::ReasoningTokens;

        assert_ne!(cost, interactions);
        assert_ne!(interactions, input);
        assert_ne!(input, output);
        assert_ne!(output, reasoning);
        assert_ne!(cost, reasoning);
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.storage_path, None);
        assert_eq!(config.refresh_interval_seconds, 60);
        assert_eq!(config.panel_metrics.len(), 5);
        assert!(config.panel_metrics.contains(&PanelMetric::Cost));
        assert!(config.panel_metrics.contains(&PanelMetric::Interactions));
        assert!(config.panel_metrics.contains(&PanelMetric::InputTokens));
        assert!(config.panel_metrics.contains(&PanelMetric::OutputTokens));
        assert!(config.panel_metrics.contains(&PanelMetric::ReasoningTokens));
        assert!(!config.use_raw_token_display);
        assert_eq!(config.display_mode, DisplayMode::Today);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_interval_too_low() {
        let config = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 0, // Too low (zero)
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        assert_eq!(
            config.validate(),
            Err(ConfigError::InvalidRefreshInterval(0))
        );
    }

    #[test]
    fn test_validate_interval_too_high() {
        let config = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 5000, // Too high
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        assert_eq!(
            config.validate(),
            Err(ConfigError::InvalidRefreshInterval(5000))
        );
    }

    #[test]
    fn test_validate_interval_at_boundaries() {
        // Minimum allowed: 1 second (with warning)
        let config_min = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 1,
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        assert_eq!(
            config_min.validate(),
            Ok(Some(ConfigWarning::LowRefreshInterval(1)))
        );

        // Just below warning threshold (59 seconds)
        let config_warning = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 59,
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        assert_eq!(
            config_warning.validate(),
            Ok(Some(ConfigWarning::LowRefreshInterval(59)))
        );

        // At warning threshold (60 seconds) - no warning
        let config_no_warning = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        assert_eq!(config_no_warning.validate(), Ok(None));

        // Maximum allowed: 3600 seconds
        let config_max = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 3600,
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        assert_eq!(config_max.validate(), Ok(None));
    }

    #[test]
    fn test_validate_refresh_interval_function() {
        // Valid intervals without warning (>= 60 seconds)
        assert_eq!(validate_refresh_interval(60), Ok(None));
        assert_eq!(validate_refresh_interval(300), Ok(None));
        assert_eq!(validate_refresh_interval(900), Ok(None));
        assert_eq!(validate_refresh_interval(3600), Ok(None));

        // Valid intervals with warning (< 60 seconds)
        assert_eq!(
            validate_refresh_interval(1),
            Ok(Some(ConfigWarning::LowRefreshInterval(1)))
        );
        assert_eq!(
            validate_refresh_interval(30),
            Ok(Some(ConfigWarning::LowRefreshInterval(30)))
        );
        assert_eq!(
            validate_refresh_interval(59),
            Ok(Some(ConfigWarning::LowRefreshInterval(59)))
        );

        // Invalid intervals (out of range)
        assert_eq!(
            validate_refresh_interval(0),
            Err(ConfigError::InvalidRefreshInterval(0))
        );
        assert_eq!(
            validate_refresh_interval(5000),
            Err(ConfigError::InvalidRefreshInterval(5000))
        );
    }

    // ===== PERSISTENCE TESTS (TDD - RED PHASE) =====

    // Helper to create test-specific app IDs to avoid test interference
    fn test_app_id(test_name: &str) -> String {
        format!("com.test.CosmicAppletOpencodeUsage.{test_name}")
    }

    #[test]
    fn test_save_config_creates_persistent_storage() {
        let app_id = test_app_id("save_creates");

        // Create a non-default config
        let config = AppConfig {
            storage_path: Some(PathBuf::from("/custom/path")),
            refresh_interval_seconds: 300,
            panel_metrics: vec![PanelMetric::Interactions, PanelMetric::Cost],
            use_raw_token_display: true,
            display_mode: DisplayMode::Month,
        };

        // Save should succeed
        let result = config.save_with_id(&app_id);
        assert!(result.is_ok(), "save() should succeed");
    }

    #[test]
    fn test_load_config_returns_defaults_when_no_config_exists() {
        let app_id = test_app_id("load_no_config");

        // Load from a fresh config (nothing saved yet)
        let loaded = AppConfig::load_with_id(&app_id);

        // Should return default config, not an error
        assert!(
            loaded.is_ok(),
            "load() should succeed even with no saved config"
        );
        let config = loaded.unwrap();
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn test_save_then_load_roundtrip() {
        let app_id = test_app_id("roundtrip");

        // Create a custom config
        let original = AppConfig {
            storage_path: Some(PathBuf::from("/test/custom/path")),
            refresh_interval_seconds: 120,
            panel_metrics: vec![PanelMetric::ReasoningTokens, PanelMetric::InputTokens],
            use_raw_token_display: true,
            display_mode: DisplayMode::AllTime,
        };

        // Save it
        original.save_with_id(&app_id).expect("save should succeed");

        // Load it back
        let loaded = AppConfig::load_with_id(&app_id).expect("load should succeed");

        // Should match the original
        assert_eq!(loaded, original);
    }

    #[test]
    fn test_save_persists_individual_fields() {
        let app_id = test_app_id("individual_fields");

        // Save a config with specific values
        let config1 = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 600,
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        config1.save_with_id(&app_id).expect("save should succeed");

        // Load and verify
        let loaded1 = AppConfig::load_with_id(&app_id).expect("load should succeed");
        assert_eq!(loaded1.refresh_interval_seconds, 600);
        assert_eq!(loaded1.panel_metrics, vec![PanelMetric::Cost]);

        // Change one field and save again
        let config2 = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 1800,
            panel_metrics: vec![PanelMetric::InputTokens],
            use_raw_token_display: true,
            display_mode: DisplayMode::Month,
        };
        config2.save_with_id(&app_id).expect("save should succeed");

        // Load and verify the change
        let loaded2 = AppConfig::load_with_id(&app_id).expect("load should succeed");
        assert_eq!(loaded2.refresh_interval_seconds, 1800);
        assert_eq!(loaded2.panel_metrics, vec![PanelMetric::InputTokens]);
        assert!(loaded2.use_raw_token_display);
    }

    // ===== DISPLAY MODE PERSISTENCE TESTS (TDD - RED PHASE) =====

    #[test]
    fn test_default_config_has_display_mode() {
        use crate::ui::state::DisplayMode;

        let config = AppConfig::default();
        assert_eq!(config.display_mode, DisplayMode::Today);
    }

    #[test]
    fn test_display_mode_roundtrip() {
        use crate::ui::state::DisplayMode;

        let app_id = test_app_id("display_mode_roundtrip");

        // Test Today mode
        let config_today = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        config_today
            .save_with_id(&app_id)
            .expect("save should succeed");
        let loaded = AppConfig::load_with_id(&app_id).expect("load should succeed");
        assert_eq!(loaded.display_mode, DisplayMode::Today);

        // Test Month mode
        let config_month = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Month,
        };
        config_month
            .save_with_id(&app_id)
            .expect("save should succeed");
        let loaded = AppConfig::load_with_id(&app_id).expect("load should succeed");
        assert_eq!(loaded.display_mode, DisplayMode::Month);

        // Test AllTime mode
        let config_alltime = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::AllTime,
        };
        config_alltime
            .save_with_id(&app_id)
            .expect("save should succeed");
        let loaded = AppConfig::load_with_id(&app_id).expect("load should succeed");
        assert_eq!(loaded.display_mode, DisplayMode::AllTime);
    }

    #[test]
    fn test_panel_metric_roundtrip() {
        let app_id = test_app_id("panel_metric_roundtrip");

        // Test single metric
        let config_single = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            panel_metrics: vec![PanelMetric::Cost],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        config_single
            .save_with_id(&app_id)
            .expect("save should succeed");
        let loaded = AppConfig::load_with_id(&app_id).expect("load should succeed");
        assert_eq!(loaded.panel_metrics, vec![PanelMetric::Cost]);

        // Test multiple metrics
        let config_multiple = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            panel_metrics: vec![
                PanelMetric::Cost,
                PanelMetric::Interactions,
                PanelMetric::ReasoningTokens,
            ],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        config_multiple
            .save_with_id(&app_id)
            .expect("save should succeed");
        let loaded = AppConfig::load_with_id(&app_id).expect("load should succeed");
        assert_eq!(
            loaded.panel_metrics,
            vec![
                PanelMetric::Cost,
                PanelMetric::Interactions,
                PanelMetric::ReasoningTokens
            ]
        );

        // Test empty Vec
        let config_empty = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            panel_metrics: vec![],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        config_empty
            .save_with_id(&app_id)
            .expect("save should succeed");
        let loaded = AppConfig::load_with_id(&app_id).expect("load should succeed");
        assert_eq!(loaded.panel_metrics, Vec::<PanelMetric>::new());

        // Test all metrics
        let config_all = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            panel_metrics: vec![
                PanelMetric::Cost,
                PanelMetric::Interactions,
                PanelMetric::InputTokens,
                PanelMetric::OutputTokens,
                PanelMetric::ReasoningTokens,
            ],
            use_raw_token_display: false,
            display_mode: DisplayMode::Today,
        };
        config_all
            .save_with_id(&app_id)
            .expect("save should succeed");
        let loaded = AppConfig::load_with_id(&app_id).expect("load should succeed");
        assert_eq!(loaded.panel_metrics.len(), 5);
        assert!(loaded.panel_metrics.contains(&PanelMetric::Cost));
        assert!(loaded.panel_metrics.contains(&PanelMetric::Interactions));
        assert!(loaded.panel_metrics.contains(&PanelMetric::InputTokens));
        assert!(loaded.panel_metrics.contains(&PanelMetric::OutputTokens));
        assert!(loaded.panel_metrics.contains(&PanelMetric::ReasoningTokens));
    }
}
