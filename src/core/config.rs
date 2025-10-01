// SPDX-License-Identifier: GPL-3.0-only

//! Configuration management for the OpenCode usage applet

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Configuration error types
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ConfigError {
    #[error("Refresh interval must be between 1 and 3600 seconds (got {0})")]
    InvalidRefreshInterval(u32),
}

/// Configuration warning types (non-blocking)
#[derive(Debug, PartialEq, Clone)]
pub enum ConfigWarning {
    /// Refresh interval is very low (< 60 seconds), may cause high CPU usage
    LowRefreshInterval(u32),
}

/// Application configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    /// Path to OpenCode storage directory (optional, defaults to ~/.local/share/opencode/storage/part)
    pub storage_path: Option<PathBuf>,
    /// Refresh interval in seconds (default: 900 = 15 minutes)
    pub refresh_interval_seconds: u32,
    /// Show today's usage next to the icon in the panel (default: false)
    pub show_today_usage: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            storage_path: None, // Will use default path from OpenCodeUsageReader
            refresh_interval_seconds: 900,
            show_today_usage: false,
        }
    }
}

impl AppConfig {
    /// Creates a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates the configuration, returning any warnings
    pub fn validate(&self) -> Result<Option<ConfigWarning>, ConfigError> {
        validate_refresh_interval(self.refresh_interval_seconds)
    }
}

/// Validates refresh interval is within acceptable range (1-3600 seconds)
/// Returns a warning (not an error) if interval is < 60 seconds
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

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.storage_path, None);
        assert_eq!(config.refresh_interval_seconds, 900);
        assert_eq!(config.show_today_usage, false);
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
            show_today_usage: false,
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
            show_today_usage: false,
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
            show_today_usage: false,
        };
        assert_eq!(
            config_min.validate(),
            Ok(Some(ConfigWarning::LowRefreshInterval(1)))
        );

        // Just below warning threshold (59 seconds)
        let config_warning = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 59,
            show_today_usage: false,
        };
        assert_eq!(
            config_warning.validate(),
            Ok(Some(ConfigWarning::LowRefreshInterval(59)))
        );

        // At warning threshold (60 seconds) - no warning
        let config_no_warning = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 60,
            show_today_usage: false,
        };
        assert_eq!(config_no_warning.validate(), Ok(None));

        // Maximum allowed: 3600 seconds
        let config_max = AppConfig {
            storage_path: None,
            refresh_interval_seconds: 3600,
            show_today_usage: false,
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
}
