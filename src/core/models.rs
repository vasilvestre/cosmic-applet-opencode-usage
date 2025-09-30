// SPDX-License-Identifier: GPL-3.0-only

//! Domain models for GitHub Copilot API responses
//!
//! This module contains the data structures used to deserialize and work with
//! GitHub Copilot usage data from the API.

use serde::{Deserialize, Serialize};

/// Represents a breakdown of usage by language and editor combination
///
/// This provides granular details about Copilot usage for a specific
/// programming language and editor pair (e.g., Rust in VSCode).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageBreakdown {
    /// Programming language (e.g., "rust", "python")
    pub language: String,
    /// Editor name (e.g., "vscode", "neovim")
    pub editor: String,
    /// Number of suggestions shown to the user
    pub suggestions_count: u32,
    /// Number of suggestions accepted by the user
    pub acceptances_count: u32,
    /// Total number of lines suggested
    pub lines_suggested: u32,
    /// Total number of lines accepted
    pub lines_accepted: u32,
}

/// Represents Copilot usage data for a specific day
///
/// This is the main data structure returned by the GitHub Copilot API,
/// containing aggregated usage metrics and detailed breakdowns.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CopilotUsage {
    /// Total number of suggestions shown across all editors/languages
    pub total_suggestions_count: u32,
    /// Total number of suggestions accepted across all editors/languages
    pub total_acceptances_count: u32,
    /// Total number of lines suggested across all editors/languages
    pub total_lines_suggested: u32,
    /// Total number of lines accepted across all editors/languages
    pub total_lines_accepted: u32,
    /// Date in YYYY-MM-DD format
    pub day: String,
    /// Detailed breakdown by language and editor
    pub breakdown: Vec<UsageBreakdown>,
}

impl CopilotUsage {
    /// Calculate the acceptance rate as a percentage (0.0-100.0)
    ///
    /// Returns the percentage of suggestions that were accepted by the user.
    /// Returns 0.0 if no suggestions were made.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmic_applet_template::core::models::CopilotUsage;
    /// let usage = CopilotUsage {
    ///     total_suggestions_count: 100,
    ///     total_acceptances_count: 50,
    ///     total_lines_suggested: 500,
    ///     total_lines_accepted: 250,
    ///     day: "2025-09-30".to_string(),
    ///     breakdown: vec![],
    /// };
    /// assert_eq!(usage.acceptance_rate(), 50.0);
    /// ```
    pub fn acceptance_rate(&self) -> f64 {
        if self.total_suggestions_count == 0 {
            0.0
        } else {
            (self.total_acceptances_count as f64 / self.total_suggestions_count as f64) * 100.0
        }
    }
}

/// Represents the seat assignment information for Copilot
///
/// Tracks the total number of seats assigned and currently active.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeatInfo {
    /// Total number of seats assigned to the organization
    pub total_seats: u32,
    /// Number of seats currently active/in use
    pub active_seats: u32,
}

impl SeatInfo {
    /// Calculate the percentage of seats currently in use
    ///
    /// Returns 0.0 if no seats are assigned.
    pub fn utilization_percentage(&self) -> f64 {
        if self.total_seats == 0 {
            0.0
        } else {
            (self.active_seats as f64 / self.total_seats as f64) * 100.0
        }
    }

    /// Check if all seats are currently in use
    pub fn is_at_capacity(&self) -> bool {
        self.active_seats >= self.total_seats
    }
}

/// Represents daily quota limits for API requests
///
/// Tracks the daily quota allocation and consumption for the GitHub API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuotaInfo {
    /// Total daily quota limit
    pub limit: u32,
    /// Amount of quota consumed so far
    pub used: u32,
    /// Amount of quota remaining
    pub remaining: u32,
}

impl QuotaInfo {
    /// Calculate the percentage of quota consumed
    pub fn usage_percentage(&self) -> f64 {
        if self.limit == 0 {
            0.0
        } else {
            (self.used as f64 / self.limit as f64) * 100.0
        }
    }

    /// Check if quota is exhausted (< 10% remaining)
    pub fn is_low(&self) -> bool {
        self.usage_percentage() > 90.0
    }

    /// Check if quota is completely exhausted
    pub fn is_exhausted(&self) -> bool {
        self.remaining == 0
    }
}

/// Represents the overall state of the applet's data
///
/// This enum captures all possible states the UI can be in:
/// loading data, displaying data, or showing an error.
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Initial state or actively fetching data
    Loading,
    /// Data successfully loaded and available
    Loaded(LoadedData),
    /// An error occurred during data fetching or processing
    Error(String),
}

/// Represents successfully loaded Copilot data
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedData {
    /// Current usage data for today
    pub usage: CopilotUsage,
    /// Seat allocation information
    pub seat_info: SeatInfo,
    /// API quota information
    pub quota_info: QuotaInfo,
}

impl AppState {
    /// Check if the state is currently loading
    pub fn is_loading(&self) -> bool {
        matches!(self, AppState::Loading)
    }

    /// Check if the state has successfully loaded data
    pub fn is_loaded(&self) -> bool {
        matches!(self, AppState::Loaded(_))
    }

    /// Check if the state represents an error
    pub fn is_error(&self) -> bool {
        matches!(self, AppState::Error(_))
    }

    /// Get the loaded data if available
    pub fn data(&self) -> Option<&LoadedData> {
        match self {
            AppState::Loaded(data) => Some(data),
            _ => None,
        }
    }

    /// Get the error message if in error state
    pub fn error_message(&self) -> Option<&str> {
        match self {
            AppState::Error(msg) => Some(msg),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copilot_usage_deserializes_from_json() {
        let json = r#"{
            "total_suggestions_count": 100,
            "total_acceptances_count": 50,
            "total_lines_suggested": 500,
            "total_lines_accepted": 250,
            "day": "2025-09-30",
            "breakdown": []
        }"#;

        let usage: CopilotUsage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.total_suggestions_count, 100);
        assert_eq!(usage.total_acceptances_count, 50);
        assert_eq!(usage.total_lines_suggested, 500);
        assert_eq!(usage.total_lines_accepted, 250);
        assert_eq!(usage.day, "2025-09-30");
    }

    #[test]
    fn test_copilot_usage_calculates_acceptance_rate() {
        let usage = CopilotUsage {
            total_suggestions_count: 100,
            total_acceptances_count: 50,
            total_lines_suggested: 500,
            total_lines_accepted: 250,
            day: "2025-09-30".to_string(),
            breakdown: vec![],
        };

        assert_eq!(usage.acceptance_rate(), 50.0);
    }

    #[test]
    fn test_copilot_usage_handles_zero_suggestions() {
        let usage = CopilotUsage {
            total_suggestions_count: 0,
            total_acceptances_count: 0,
            total_lines_suggested: 0,
            total_lines_accepted: 0,
            day: "2025-09-30".to_string(),
            breakdown: vec![],
        };

        assert_eq!(usage.acceptance_rate(), 0.0);
    }

    #[test]
    fn test_usage_breakdown_deserializes_from_json() {
        let json = r#"{
            "language": "rust",
            "editor": "vscode",
            "suggestions_count": 50,
            "acceptances_count": 30,
            "lines_suggested": 200,
            "lines_accepted": 120
        }"#;

        let breakdown: UsageBreakdown = serde_json::from_str(json).unwrap();
        assert_eq!(breakdown.language, "rust");
        assert_eq!(breakdown.editor, "vscode");
        assert_eq!(breakdown.suggestions_count, 50);
        assert_eq!(breakdown.acceptances_count, 30);
    }

    // Tests for SeatInfo
    #[test]
    fn test_seat_info_calculates_utilization() {
        let seat_info = SeatInfo {
            total_seats: 100,
            active_seats: 75,
        };

        assert_eq!(seat_info.utilization_percentage(), 75.0);
    }

    #[test]
    fn test_seat_info_handles_zero_seats() {
        let seat_info = SeatInfo {
            total_seats: 0,
            active_seats: 0,
        };

        assert_eq!(seat_info.utilization_percentage(), 0.0);
    }

    #[test]
    fn test_seat_info_detects_at_capacity() {
        let at_capacity = SeatInfo {
            total_seats: 100,
            active_seats: 100,
        };
        assert!(at_capacity.is_at_capacity());

        let over_capacity = SeatInfo {
            total_seats: 100,
            active_seats: 105,
        };
        assert!(over_capacity.is_at_capacity());

        let not_at_capacity = SeatInfo {
            total_seats: 100,
            active_seats: 75,
        };
        assert!(!not_at_capacity.is_at_capacity());
    }

    // Tests for QuotaInfo
    #[test]
    fn test_quota_info_calculates_usage_percentage() {
        let quota = QuotaInfo {
            limit: 5000,
            used: 1250,
            remaining: 3750,
        };

        assert_eq!(quota.usage_percentage(), 25.0);
    }

    #[test]
    fn test_quota_info_detects_low_quota() {
        let low_quota = QuotaInfo {
            limit: 5000,
            used: 4600,
            remaining: 400,
        };
        assert!(low_quota.is_low());

        let normal_quota = QuotaInfo {
            limit: 5000,
            used: 2500,
            remaining: 2500,
        };
        assert!(!normal_quota.is_low());
    }

    #[test]
    fn test_quota_info_detects_exhausted_quota() {
        let exhausted = QuotaInfo {
            limit: 5000,
            used: 5000,
            remaining: 0,
        };
        assert!(exhausted.is_exhausted());

        let available = QuotaInfo {
            limit: 5000,
            used: 4999,
            remaining: 1,
        };
        assert!(!available.is_exhausted());
    }

    #[test]
    fn test_quota_info_deserializes_from_json() {
        let json = r#"{
            "limit": 5000,
            "used": 1250,
            "remaining": 3750
        }"#;

        let quota: QuotaInfo = serde_json::from_str(json).unwrap();
        assert_eq!(quota.limit, 5000);
        assert_eq!(quota.used, 1250);
        assert_eq!(quota.remaining, 3750);
    }

    // Tests for AppState
    #[test]
    fn test_app_state_loading() {
        let state = AppState::Loading;
        
        assert!(state.is_loading());
        assert!(!state.is_loaded());
        assert!(!state.is_error());
        assert!(state.data().is_none());
        assert!(state.error_message().is_none());
    }

    #[test]
    fn test_app_state_loaded() {
        let usage = CopilotUsage {
            total_suggestions_count: 100,
            total_acceptances_count: 50,
            total_lines_suggested: 500,
            total_lines_accepted: 250,
            day: "2025-09-30".to_string(),
            breakdown: vec![],
        };

        let seat_info = SeatInfo {
            total_seats: 100,
            active_seats: 75,
        };

        let quota_info = QuotaInfo {
            limit: 5000,
            used: 1250,
            remaining: 3750,
        };

        let loaded_data = LoadedData {
            usage,
            seat_info,
            quota_info,
        };

        let state = AppState::Loaded(loaded_data.clone());
        
        assert!(!state.is_loading());
        assert!(state.is_loaded());
        assert!(!state.is_error());
        assert!(state.data().is_some());
        assert_eq!(state.data().unwrap().usage.acceptance_rate(), 50.0);
        assert!(state.error_message().is_none());
    }

    #[test]
    fn test_app_state_error() {
        let error_msg = "Network connection failed".to_string();
        let state = AppState::Error(error_msg.clone());
        
        assert!(!state.is_loading());
        assert!(!state.is_loaded());
        assert!(state.is_error());
        assert!(state.data().is_none());
        assert_eq!(state.error_message(), Some(error_msg.as_str()));
    }

    #[test]
    fn test_loaded_data_contains_all_components() {
        let usage = CopilotUsage {
            total_suggestions_count: 100,
            total_acceptances_count: 50,
            total_lines_suggested: 500,
            total_lines_accepted: 250,
            day: "2025-09-30".to_string(),
            breakdown: vec![],
        };

        let seat_info = SeatInfo {
            total_seats: 100,
            active_seats: 75,
        };

        let quota_info = QuotaInfo {
            limit: 5000,
            used: 1250,
            remaining: 3750,
        };

        let loaded_data = LoadedData {
            usage: usage.clone(),
            seat_info: seat_info.clone(),
            quota_info: quota_info.clone(),
        };

        assert_eq!(loaded_data.usage, usage);
        assert_eq!(loaded_data.seat_info, seat_info);
        assert_eq!(loaded_data.quota_info, quota_info);
    }
}

