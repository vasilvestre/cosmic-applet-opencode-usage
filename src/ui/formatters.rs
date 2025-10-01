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

/// Format tokens compactly for panel display (e.g., "1.2k", "15M")
pub fn format_tokens_compact(tokens: u64) -> String {
    if tokens < 1_000 {
        tokens.to_string()
    } else if tokens < 1_000_000 {
        let k = tokens as f64 / 1_000.0;
        let rounded = k.round();
        if k >= 10.0 || (k - rounded).abs() < 0.01 {
            format!("{}k", rounded as u64)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        let m = tokens as f64 / 1_000_000.0;
        let rounded = m.round();
        if m >= 10.0 || (m - rounded).abs() < 0.01 {
            format!("{}M", rounded as u64)
        } else {
            format!("{:.1}M", m)
        }
    }
}

/// Format panel display with cost and total tokens (e.g., "$1.2 | 15k")
pub fn format_panel_display(usage: &UsageMetrics) -> String {
    let cost = format_cost_compact(usage.total_cost);
    let total_tokens = usage.total_input_tokens + usage.total_output_tokens;
    let tokens = format_tokens_compact(total_tokens);
    format!("{} | {}", cost, tokens)
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

    #[test]
    fn test_format_tokens_compact_hundreds() {
        assert_eq!(format_tokens_compact(123), "123");
        assert_eq!(format_tokens_compact(999), "999");
    }

    #[test]
    fn test_format_tokens_compact_thousands() {
        assert_eq!(format_tokens_compact(1_000), "1k");
        assert_eq!(format_tokens_compact(1_234), "1.2k");
        assert_eq!(format_tokens_compact(9_999), "10k");
        assert_eq!(format_tokens_compact(15_678), "16k");
    }

    #[test]
    fn test_format_tokens_compact_millions() {
        assert_eq!(format_tokens_compact(1_000_000), "1M");
        assert_eq!(format_tokens_compact(1_234_567), "1.2M");
        assert_eq!(format_tokens_compact(9_999_999), "10M");
    }

    #[test]
    fn test_format_tokens_compact_zero() {
        assert_eq!(format_tokens_compact(0), "0");
    }

    #[test]
    fn test_format_panel_display_small_values() {
        let usage = UsageMetrics {
            total_input_tokens: 100,
            total_output_tokens: 50,
            total_reasoning_tokens: 0,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 0.05,
            interaction_count: 1,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(format_panel_display(&usage), "$0.05 | 150");
    }

    #[test]
    fn test_format_panel_display_medium_values() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 0,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(format_panel_display(&usage), "$1.2 | 15k");
    }

    #[test]
    fn test_format_panel_display_large_values() {
        let usage = UsageMetrics {
            total_input_tokens: 500_000,
            total_output_tokens: 250_000,
            total_reasoning_tokens: 0,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 12.50,
            interaction_count: 25,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(format_panel_display(&usage), "$12 | 750k");
    }
}
