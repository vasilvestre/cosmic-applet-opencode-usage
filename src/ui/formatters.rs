// SPDX-License-Identifier: GPL-3.0-only

//! Number and text formatting utilities for the UI

use chrono::{DateTime, Utc};

const THOUSAND: u64 = 1_000;
const MILLION: u64 = 1_000_000;
const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Formats a number for compact display in the panel
///
/// # Format Rules
/// - 0-999: Display as-is (e.g., "42", "999")
/// - 1,000-999,999: Display in K format with one decimal (e.g., "1.5K", "42.3K")
/// - 1,000,000+: Display in M format with one decimal (e.g., "1.2M", "15.7M")
///
/// # Examples
/// ```
/// use cosmic_applet_copilot_quota_tracker::ui::formatters::format_number;
///
/// assert_eq!(format_number(42), "42");
/// assert_eq!(format_number(1500), "1.5K");
/// assert_eq!(format_number(1234567), "1.2M");
/// ```
pub fn format_number(n: u64) -> String {
    if n < THOUSAND {
        n.to_string()
    } else if n < MILLION {
        let k = n as f64 / THOUSAND as f64;
        format!("{:.1}K", k)
    } else {
        let m = n as f64 / MILLION as f64;
        format!("{:.1}M", m)
    }
}

/// Formats a tooltip text showing last update time or "no data" message
///
/// # Arguments
/// * `last_update` - Optional timestamp of the last data update
///
/// # Returns
/// - If `Some(datetime)`: "Last updated: YYYY-MM-DD HH:MM:SS"
/// - If `None`: "No data available"
///
/// # Examples
/// ```
/// use cosmic_applet_copilot_quota_tracker::ui::formatters::format_tooltip;
/// use chrono::{DateTime, Utc, TimeZone};
///
/// let dt = Utc.with_ymd_and_hms(2025, 9, 30, 14, 30, 0).unwrap();
/// assert_eq!(format_tooltip(Some(dt)), "Last updated: 2025-09-30 14:30:00");
/// assert_eq!(format_tooltip(None), "No data available");
/// ```
pub fn format_tooltip(last_update: Option<DateTime<Utc>>) -> String {
    match last_update {
        Some(dt) => format!("Last updated: {}", dt.format(DATETIME_FORMAT)),
        None => "No data available".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // Tests for format_number()
    #[test]
    fn test_format_number_under_thousand() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(42), "42");
        assert_eq!(format_number(999), "999");
    }

    #[test]
    fn test_format_number_thousands() {
        assert_eq!(format_number(1000), "1.0K");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(42_300), "42.3K");
        assert_eq!(format_number(999_999), "1000.0K");
    }

    #[test]
    fn test_format_number_millions() {
        assert_eq!(format_number(1_000_000), "1.0M");
        assert_eq!(format_number(1_234_567), "1.2M");
        assert_eq!(format_number(15_700_000), "15.7M");
        assert_eq!(format_number(999_999_999), "1000.0M");
    }

    // Tests for format_tooltip()
    #[test]
    fn test_format_tooltip_with_timestamp() {
        let dt = Utc.with_ymd_and_hms(2025, 9, 30, 14, 30, 0).unwrap();
        assert_eq!(
            format_tooltip(Some(dt)),
            "Last updated: 2025-09-30 14:30:00"
        );
    }

    #[test]
    fn test_format_tooltip_no_data() {
        assert_eq!(format_tooltip(None), "No data available");
    }

    #[test]
    fn test_format_tooltip_recent_time() {
        // Test with current time to ensure formatting works
        let now = Utc::now();
        let result = format_tooltip(Some(now));
        
        // Should start with "Last updated: " and have the right format
        assert!(result.starts_with("Last updated: "));
        assert!(result.contains(&now.format("%Y-%m-%d").to_string()));
    }
}
