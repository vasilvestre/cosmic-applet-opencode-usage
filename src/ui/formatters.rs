// SPDX-License-Identifier: GPL-3.0-only

//! UI formatting utilities

use crate::core::opencode::UsageMetrics;
use chrono::{DateTime, Utc};

/// Format a number with thousand separators
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();
    
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    
    result
}

/// Format cost in dollars
pub fn format_cost(cost: f64) -> String {
    format!("${:.2}", cost)
}

/// Format cost compactly for panel display
pub fn format_cost_compact(cost: f64) -> String {
    if cost >= 10.0 {
        format!("${:.0}", cost)
    } else if cost >= 1.0 {
        format!("${:.1}", cost)
    } else {
        format!("${:.2}", cost)
    }
}

/// Get the primary metric to display (total cost)
pub fn get_primary_metric(usage: &UsageMetrics) -> u64 {
    // Convert cost to cents for display as integer
    (usage.total_cost * 100.0) as u64
}

/// Format tooltip with last update timestamp
pub fn format_tooltip(last_update: Option<DateTime<Utc>>) -> String {
    match last_update {
        Some(timestamp) => {
            format!("Last updated: {}", timestamp.format("%Y-%m-%d %H:%M:%S"))
        }
        None => "No data available".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number_small() {
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_format_number_thousands() {
        assert_eq!(format_number(1234), "1,234");
    }

    #[test]
    fn test_format_number_millions() {
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_cost() {
        assert_eq!(format_cost(12.5), "$12.50");
        assert_eq!(format_cost(0.99), "$0.99");
        assert_eq!(format_cost(1234.567), "$1234.57");
    }

    #[test]
    fn test_format_cost_compact_small() {
        assert_eq!(format_cost_compact(0.05), "$0.05");
        assert_eq!(format_cost_compact(0.99), "$0.99");
    }

    #[test]
    fn test_format_cost_compact_medium() {
        assert_eq!(format_cost_compact(1.5), "$1.5");
        assert_eq!(format_cost_compact(5.99), "$6.0");
        assert_eq!(format_cost_compact(9.45), "$9.4");
    }

    #[test]
    fn test_format_cost_compact_large() {
        assert_eq!(format_cost_compact(10.0), "$10");
        assert_eq!(format_cost_compact(12.5), "$12");
        assert_eq!(format_cost_compact(125.67), "$126");
    }

    #[test]
    fn test_format_tooltip_with_data() {
        let timestamp = chrono::Utc::now();
        let tooltip = format_tooltip(Some(timestamp));
        assert!(tooltip.starts_with("Last updated: "));
    }

    #[test]
    fn test_format_tooltip_without_data() {
        let tooltip = format_tooltip(None);
        assert_eq!(tooltip, "No data available");
    }
}
