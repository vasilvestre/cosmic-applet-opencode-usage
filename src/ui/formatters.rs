// SPDX-License-Identifier: GPL-3.0-only

//! Number and text formatting utilities for the UI

const THOUSAND: u64 = 1_000;
const MILLION: u64 = 1_000_000;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
