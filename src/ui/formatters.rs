// SPDX-License-Identifier: GPL-3.0-only

//! UI formatting utilities

use crate::core::config::PanelMetric;
use crate::core::opencode::UsageMetrics;
use chrono::{DateTime, Utc};

/// Format a number with locale-aware thousand separators
/// Uses the system locale to determine the appropriate separator
#[must_use]
pub fn format_number(n: u64) -> String {
    format_number_locale(n)
}

/// Format cost in dollars
#[must_use]
pub fn format_cost(cost: f64) -> String {
    format!("${cost:.2}")
}

/// Format cost compactly for panel display
#[must_use]
pub fn format_cost_compact(cost: f64) -> String {
    if cost >= 10.0 {
        format!("${cost:.0}")
    } else if cost >= 1.0 {
        format!("${cost:.1}")
    } else {
        format!("${cost:.2}")
    }
}

/// Format tokens compactly for panel display (e.g., "1.2k", "15M")
#[must_use]
pub fn format_tokens_compact(tokens: u64) -> String {
    if tokens < 1_000 {
        tokens.to_string()
    } else if tokens < 1_000_000 {
        #[allow(clippy::cast_precision_loss)]
        let k = tokens as f64 / 1_000.0;
        let rounded = k.round();
        if k >= 10.0 || (k - rounded).abs() < 0.01 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let rounded_u64 = rounded as u64;
            format!("{rounded_u64}k")
        } else {
            format!("{k:.1}k")
        }
    } else {
        #[allow(clippy::cast_precision_loss)]
        let m = tokens as f64 / 1_000_000.0;
        let rounded = m.round();
        if m >= 10.0 || (m - rounded).abs() < 0.01 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let rounded_u64 = rounded as u64;
            format!("{rounded_u64}M")
        } else {
            format!("{m:.1}M")
        }
    }
}

/// Format tokens as raw numbers without K/M suffixes, using locale-aware thousand separators
/// (e.g., "1,000" in US, "1 000" in FR, "1.000" in DE)
#[must_use]
pub fn format_tokens_raw(tokens: u64) -> String {
    format_number_locale(tokens)
}

/// Format a number with locale-aware thousand separators
/// Uses the system locale to determine the appropriate separator
#[must_use]
pub fn format_number_locale(n: u64) -> String {
    use num_format::{Locale, ToFormattedString};

    // Try to get system locale, fallback to US English if unavailable
    match num_format::SystemLocale::default() {
        Ok(locale) => n.to_formatted_string(&locale),
        Err(_) => n.to_formatted_string(&Locale::en),
    }
}

/// Format panel display ultra-compact for narrow panels (e.g., "15k/$1.2")
#[must_use]
pub fn format_panel_display(usage: &UsageMetrics) -> String {
    let cost = format_cost_compact(usage.total_cost);
    let total_tokens = usage.total_input_tokens + usage.total_output_tokens;
    let tokens = format_tokens_compact(total_tokens);
    format!("{tokens}/{cost}")
}

/// Format comprehensive panel display with all metrics (e.g., "$1.2 | 3x | 10k/5k/2k")
/// Format: Cost | Interactions | InputTokens/OutputTokens/ReasoningTokens
#[must_use]
pub fn format_panel_display_detailed(usage: &UsageMetrics) -> String {
    let cost = format_cost_compact(usage.total_cost);
    let interactions = usage.interaction_count;
    let input_tokens = format_tokens_compact(usage.total_input_tokens);
    let output_tokens = format_tokens_compact(usage.total_output_tokens);
    let reasoning_tokens = format_tokens_compact(usage.total_reasoning_tokens);
    format!("{cost} | {interactions}x | {input_tokens}/{output_tokens}/{reasoning_tokens}")
}

/// Format comprehensive panel display with raw token values (e.g., "$1.2 | 3x | 10000/5000/2000")
/// Format: Cost | Interactions | InputTokens/OutputTokens/ReasoningTokens (no K/M suffixes)
#[must_use]
pub fn format_panel_display_detailed_raw(usage: &UsageMetrics) -> String {
    let cost = format_cost_compact(usage.total_cost);
    let interactions = usage.interaction_count;
    let input_tokens = format_tokens_raw(usage.total_input_tokens);
    let output_tokens = format_tokens_raw(usage.total_output_tokens);
    let reasoning_tokens = format_tokens_raw(usage.total_reasoning_tokens);
    format!("{cost} | {interactions}x | {input_tokens}/{output_tokens}/{reasoning_tokens}")
}

/// Format only cost for panel display (e.g., "$1.2")
#[must_use]
pub fn format_panel_cost_only(usage: &UsageMetrics) -> String {
    format_cost_compact(usage.total_cost)
}

/// Format only interaction count for panel display (e.g., "5x")
#[must_use]
pub fn format_panel_interactions_only(usage: &UsageMetrics) -> String {
    format!("{}x", usage.interaction_count)
}

/// Format only input tokens for panel display (e.g., "10k")
#[must_use]
pub fn format_panel_input_tokens_only(usage: &UsageMetrics) -> String {
    format_tokens_compact(usage.total_input_tokens)
}

/// Format only output tokens for panel display (e.g., "5k")
#[must_use]
pub fn format_panel_output_tokens_only(usage: &UsageMetrics) -> String {
    format_tokens_compact(usage.total_output_tokens)
}

/// Format only reasoning tokens for panel display (e.g., "2k")
#[must_use]
pub fn format_panel_reasoning_tokens_only(usage: &UsageMetrics) -> String {
    format_tokens_compact(usage.total_reasoning_tokens)
}

/// Format only input tokens with raw numbers for panel display (e.g., "10,000")
#[must_use]
pub fn format_panel_input_tokens_only_raw(usage: &UsageMetrics) -> String {
    format_tokens_raw(usage.total_input_tokens)
}

/// Format only output tokens with raw numbers for panel display (e.g., "5,000")
#[must_use]
pub fn format_panel_output_tokens_only_raw(usage: &UsageMetrics) -> String {
    format_tokens_raw(usage.total_output_tokens)
}

/// Format only reasoning tokens with raw numbers for panel display (e.g., "2,000")
#[must_use]
pub fn format_panel_reasoning_tokens_only_raw(usage: &UsageMetrics) -> String {
    format_tokens_raw(usage.total_reasoning_tokens)
}

/// Display order for panel metrics (fixed order regardless of selection order)
/// Cost | Interactions | `InputTokens` | `OutputTokens` | `ReasoningTokens`
const METRIC_DISPLAY_ORDER: [PanelMetric; 5] = [
    PanelMetric::Cost,
    PanelMetric::Interactions,
    PanelMetric::InputTokens,
    PanelMetric::OutputTokens,
    PanelMetric::ReasoningTokens,
];

/// Format panel metric based on the selected metric type
///
/// This dispatcher function routes to the appropriate formatter based on the `PanelMetric` enum.
/// The `use_raw` parameter determines whether to use compact (K/M) or raw (with separators) token display.
///
/// # Arguments
/// * `usage` - The usage metrics to format
/// * `metric` - The panel metric type to display
/// * `use_raw` - Whether to use raw token display (ignored for Cost and Interactions)
///
/// # Returns
/// * Formatted string for the selected metric
#[must_use]
pub fn format_panel_metric(usage: &UsageMetrics, metric: PanelMetric, use_raw: bool) -> String {
    match metric {
        PanelMetric::Cost => format_panel_cost_only(usage),
        PanelMetric::Interactions => format_panel_interactions_only(usage),
        PanelMetric::InputTokens => {
            if use_raw {
                format_panel_input_tokens_only_raw(usage)
            } else {
                format_panel_input_tokens_only(usage)
            }
        }
        PanelMetric::OutputTokens => {
            if use_raw {
                format_panel_output_tokens_only_raw(usage)
            } else {
                format_panel_output_tokens_only(usage)
            }
        }
        PanelMetric::ReasoningTokens => {
            if use_raw {
                format_panel_reasoning_tokens_only_raw(usage)
            } else {
                format_panel_reasoning_tokens_only(usage)
            }
        }
    }
}

/// Format multiple panel metrics in a fixed order
///
/// Format: "$1.23 5x IT: 10k OT: 5k RT: 2k"
/// - Cost: "$X.XX" (no prefix)
/// - Interactions: "Xx" (no prefix)
/// - `InputTokens`: "IT: `XXk`" (with prefix)
/// - `OutputTokens`: "OT: `XXk`" (with prefix)
/// - `ReasoningTokens`: "RT: `XXk`" (with prefix)
///
/// The metrics are displayed in a fixed order (Cost, Interactions, `InputTokens`, `OutputTokens`, `ReasoningTokens`)
/// regardless of the order they appear in the input vector. Metrics not present in the vector are skipped.
///
/// # Arguments
/// * `usage` - The usage metrics to format
/// * `metrics` - Vector of panel metrics to display (order doesn't matter, will be reordered)
/// * `use_raw` - Whether to use raw token display (ignored for Cost and Interactions)
///
/// # Returns
/// * Formatted string with selected metrics separated by spaces, or empty string if metrics is empty
#[must_use]
pub fn format_multiple_panel_metrics(
    usage: &UsageMetrics,
    metrics: &[PanelMetric],
    use_raw: bool,
) -> String {
    if metrics.is_empty() {
        return String::new();
    }

    // Convert to a set-like structure for O(1) lookup
    let metric_set: std::collections::HashSet<PanelMetric> = metrics.iter().copied().collect();

    // Format metrics in display order
    let formatted_metrics: Vec<String> = METRIC_DISPLAY_ORDER
        .iter()
        .filter(|m| metric_set.contains(m))
        .map(|metric| {
            let value = format_panel_metric(usage, *metric, use_raw);
            match metric {
                PanelMetric::Cost | PanelMetric::Interactions => value,
                PanelMetric::InputTokens => format!("IT: {value}"),
                PanelMetric::OutputTokens => format!("OT: {value}"),
                PanelMetric::ReasoningTokens => format!("RT: {value}"),
            }
        })
        .collect();

    formatted_metrics.join(" ")
}

/// Get the primary metric to display (total cost)
#[must_use]
pub fn get_primary_metric(usage: &UsageMetrics) -> u64 {
    // Convert cost to cents for display as integer
    // The cost is always positive and should be within u64 range
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let cents = (usage.total_cost * 100.0) as u64;
    cents
}

/// Format tooltip with last update timestamp
#[must_use]
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

    // Note: These tests verify that locale-aware formatting works.
    // The exact output depends on the system locale, so we test that:
    // 1. Numbers under 1000 have no separators
    // 2. Numbers 1000+ have some separator character
    // 3. The numeric value is preserved (all separators can be removed to get original)

    #[test]
    fn test_format_number_small() {
        let result = format_number(123);
        // Small numbers should not have separators
        assert_eq!(result, "123");
    }

    #[test]
    fn test_format_number_thousands() {
        let result = format_number(1234);
        eprintln!(
            "DEBUG: format_number(1234) = '{}' (len={})",
            result,
            result.len()
        );
        // The length will depend on locale:
        // - English: "1,234" (5 chars: comma separator)
        // - French: "1 234" (5 chars: space separator - actually might be non-breaking space = 3 bytes)
        // - German: "1.234" (5 chars: period separator)
        // We just verify it has the right digits
        assert!(result.len() >= 4); // At least the 4 digits
                                    // Should contain the digits 1, 2, 3, 4
        assert!(result.contains('1'));
        assert!(result.contains('2'));
        assert!(result.contains('3'));
        assert!(result.contains('4'));
    }

    #[test]
    fn test_format_number_millions() {
        let result = format_number(1_234_567);
        eprintln!(
            "DEBUG: format_number(1234567) = '{}' (len={})",
            result,
            result.len()
        );
        // Length depends on locale and number of separators
        // We just verify the digits are all present
        assert!(result.len() >= 7); // At least the 7 digits
                                    // Removing non-digits should give us the original number
        let digits_only: String = result.chars().filter(char::is_ascii_digit).collect();
        assert_eq!(digits_only, "1234567");
    }

    #[test]
    fn test_format_number_locale_consistency() {
        // Test that format_number and format_number_locale produce the same output
        assert_eq!(format_number(1000), format_number_locale(1000));
        assert_eq!(format_number(1_234_567), format_number_locale(1_234_567));
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
        assert_eq!(format_panel_display(&usage), "150/$0.05");
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
        assert_eq!(format_panel_display(&usage), "15k/$1.2");
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
        assert_eq!(format_panel_display(&usage), "750k/$12");
    }

    #[test]
    fn test_format_panel_display_detailed_small() {
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
        assert_eq!(
            format_panel_display_detailed(&usage),
            "$0.05 | 1x | 100/50/0"
        );
    }

    #[test]
    fn test_format_panel_display_detailed_medium() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 0,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 15,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(
            format_panel_display_detailed(&usage),
            "$1.2 | 15x | 10k/5k/0"
        );
    }

    #[test]
    fn test_format_panel_display_detailed_large() {
        let usage = UsageMetrics {
            total_input_tokens: 25_000_000,
            total_output_tokens: 10_000_000,
            total_reasoning_tokens: 0,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 125.50,
            interaction_count: 1234,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(
            format_panel_display_detailed(&usage),
            "$126 | 1234x | 25M/10M/0"
        );
    }

    #[test]
    fn test_format_panel_display_detailed_with_reasoning() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 15,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(
            format_panel_display_detailed(&usage),
            "$1.2 | 15x | 10k/5k/2k"
        );
    }

    #[test]
    fn test_format_tokens_raw_small() {
        assert_eq!(format_tokens_raw(100), "100");
        assert_eq!(format_tokens_raw(999), "999");
    }

    #[test]
    #[allow(clippy::similar_names)] // Test names intentionally similar (1k, 10k, 999k)
    fn test_format_tokens_raw_thousands() {
        // These should have locale-aware separators
        let result_1k = format_tokens_raw(1_000);
        let result_10k = format_tokens_raw(10_500);
        let result_999k = format_tokens_raw(999_999);

        // Verify the numeric content is preserved
        assert_eq!(
            result_1k
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>(),
            "1000"
        );
        assert_eq!(
            result_10k
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>(),
            "10500"
        );
        assert_eq!(
            result_999k
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>(),
            "999999"
        );
    }

    #[test]
    fn test_format_tokens_raw_millions() {
        let result_1m = format_tokens_raw(1_000_000);
        let result_25m = format_tokens_raw(25_000_000);

        // Verify the numeric content is preserved
        assert_eq!(
            result_1m
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>(),
            "1000000"
        );
        assert_eq!(
            result_25m
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>(),
            "25000000"
        );
    }

    #[test]
    fn test_format_panel_display_detailed_raw_small() {
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
        let result = format_panel_display_detailed_raw(&usage);
        // Small values should not have separators
        assert_eq!(result, "$0.05 | 1x | 100/50/0");
    }

    #[test]
    fn test_format_panel_display_detailed_raw_large() {
        let usage = UsageMetrics {
            total_input_tokens: 25_000_000,
            total_output_tokens: 10_000_000,
            total_reasoning_tokens: 0,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 125.50,
            interaction_count: 1234,
            timestamp: std::time::SystemTime::now(),
        };
        let result = format_panel_display_detailed_raw(&usage);
        eprintln!("DEBUG: format_panel_display_detailed_raw = '{result}'");
        // Should contain the cost and interaction count
        assert!(result.starts_with("$126 | 1234x | "));
        // Should contain all the digits for the token counts (with possible separators)
        let digits_only: String = result.chars().filter(char::is_ascii_digit).collect();
        eprintln!("DEBUG: digits_only = '{digits_only}'");
        // The digits should include: 126, 1234, 25000000, 10000000
        assert!(digits_only.contains("25000000"));
        assert!(digits_only.contains("10000000"));
    }

    // Individual metric formatter tests
    #[test]
    fn test_format_panel_cost_only() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(format_panel_cost_only(&usage), "$1.2");
    }

    #[test]
    fn test_format_panel_interactions_only() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(format_panel_interactions_only(&usage), "5x");
    }

    #[test]
    fn test_format_panel_input_tokens_only() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(format_panel_input_tokens_only(&usage), "10k");
    }

    #[test]
    fn test_format_panel_output_tokens_only() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(format_panel_output_tokens_only(&usage), "5k");
    }

    #[test]
    fn test_format_panel_reasoning_tokens_only() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };
        assert_eq!(format_panel_reasoning_tokens_only(&usage), "2k");
    }

    // Test raw token display variants
    #[test]
    fn test_format_panel_input_tokens_only_raw() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };
        let result = format_panel_input_tokens_only_raw(&usage);
        // Should contain the digits 10000 (possibly with separators)
        let digits_only: String = result.chars().filter(char::is_ascii_digit).collect();
        assert_eq!(digits_only, "10000");
    }

    #[test]
    fn test_format_panel_output_tokens_only_raw() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };
        let result = format_panel_output_tokens_only_raw(&usage);
        let digits_only: String = result.chars().filter(char::is_ascii_digit).collect();
        assert_eq!(digits_only, "5000");
    }

    #[test]
    fn test_format_panel_reasoning_tokens_only_raw() {
        let usage = UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };
        let result = format_panel_reasoning_tokens_only_raw(&usage);
        let digits_only: String = result.chars().filter(char::is_ascii_digit).collect();
        assert_eq!(digits_only, "2000");
    }

    // Dispatcher function tests
    #[test]
    fn test_format_panel_metric_cost() {
        let usage = create_test_usage();
        assert_eq!(
            format_panel_metric(&usage, PanelMetric::Cost, false),
            "$1.2"
        );
    }

    #[test]
    fn test_format_panel_metric_interactions() {
        let usage = create_test_usage();
        assert_eq!(
            format_panel_metric(&usage, PanelMetric::Interactions, false),
            "5x"
        );
    }

    #[test]
    fn test_format_panel_metric_input_tokens() {
        let usage = create_test_usage();
        assert_eq!(
            format_panel_metric(&usage, PanelMetric::InputTokens, false),
            "10k"
        );
    }

    #[test]
    fn test_format_panel_metric_output_tokens() {
        let usage = create_test_usage();
        assert_eq!(
            format_panel_metric(&usage, PanelMetric::OutputTokens, false),
            "5k"
        );
    }

    #[test]
    fn test_format_panel_metric_reasoning_tokens() {
        let usage = create_test_usage();
        assert_eq!(
            format_panel_metric(&usage, PanelMetric::ReasoningTokens, false),
            "2k"
        );
    }

    #[test]
    fn test_format_panel_metric_input_tokens_raw() {
        let usage = create_test_usage();
        let result = format_panel_metric(&usage, PanelMetric::InputTokens, true);
        let digits_only: String = result.chars().filter(char::is_ascii_digit).collect();
        assert_eq!(digits_only, "10000");
    }

    #[test]
    fn test_format_panel_metric_output_tokens_raw() {
        let usage = create_test_usage();
        let result = format_panel_metric(&usage, PanelMetric::OutputTokens, true);
        let digits_only: String = result.chars().filter(char::is_ascii_digit).collect();
        assert_eq!(digits_only, "5000");
    }

    #[test]
    fn test_format_panel_metric_reasoning_tokens_raw() {
        let usage = create_test_usage();
        let result = format_panel_metric(&usage, PanelMetric::ReasoningTokens, true);
        let digits_only: String = result.chars().filter(char::is_ascii_digit).collect();
        assert_eq!(digits_only, "2000");
    }

    // Helper function for tests
    fn create_test_usage() -> UsageMetrics {
        UsageMetrics {
            total_input_tokens: 10_000,
            total_output_tokens: 5_000,
            total_reasoning_tokens: 2_000,
            total_cache_write_tokens: 0,
            total_cache_read_tokens: 0,
            total_cost: 1.23,
            interaction_count: 5,
            timestamp: std::time::SystemTime::now(),
        }
    }

    // ===== MULTI-METRIC FORMATTER TESTS (TDD - RED PHASE) =====

    #[test]
    fn test_format_multiple_panel_metrics_empty() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(&usage, &[], false);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_multiple_panel_metrics_single_cost() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(&usage, &[PanelMetric::Cost], false);
        assert_eq!(result, "$1.2");
    }

    #[test]
    fn test_format_multiple_panel_metrics_single_interactions() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(&usage, &[PanelMetric::Interactions], false);
        assert_eq!(result, "5x");
    }

    #[test]
    fn test_format_multiple_panel_metrics_single_input_tokens() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(&usage, &[PanelMetric::InputTokens], false);
        assert_eq!(result, "IT: 10k");
    }

    #[test]
    fn test_format_multiple_panel_metrics_single_output_tokens() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(&usage, &[PanelMetric::OutputTokens], false);
        assert_eq!(result, "OT: 5k");
    }

    #[test]
    fn test_format_multiple_panel_metrics_single_reasoning_tokens() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(&usage, &[PanelMetric::ReasoningTokens], false);
        assert_eq!(result, "RT: 2k");
    }

    #[test]
    fn test_format_multiple_panel_metrics_all_metrics() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(
            &usage,
            &[
                PanelMetric::Cost,
                PanelMetric::Interactions,
                PanelMetric::InputTokens,
                PanelMetric::OutputTokens,
                PanelMetric::ReasoningTokens,
            ],
            false,
        );
        assert_eq!(result, "$1.2 5x IT: 10k OT: 5k RT: 2k");
    }

    #[test]
    fn test_format_multiple_panel_metrics_cost_and_interactions() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(
            &usage,
            &[PanelMetric::Cost, PanelMetric::Interactions],
            false,
        );
        assert_eq!(result, "$1.2 5x");
    }

    #[test]
    fn test_format_multiple_panel_metrics_tokens_only() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(
            &usage,
            &[
                PanelMetric::InputTokens,
                PanelMetric::OutputTokens,
                PanelMetric::ReasoningTokens,
            ],
            false,
        );
        assert_eq!(result, "IT: 10k OT: 5k RT: 2k");
    }

    #[test]
    fn test_format_multiple_panel_metrics_fixed_order_regardless_of_input() {
        let usage = create_test_usage();
        // Test that order is always Cost -> Interactions -> InputTokens -> OutputTokens -> ReasoningTokens
        // regardless of input order
        let result1 = format_multiple_panel_metrics(
            &usage,
            &[PanelMetric::ReasoningTokens, PanelMetric::Cost],
            false,
        );
        let result2 = format_multiple_panel_metrics(
            &usage,
            &[PanelMetric::Cost, PanelMetric::ReasoningTokens],
            false,
        );
        assert_eq!(result1, result2);
        assert_eq!(result1, "$1.2 RT: 2k");
    }

    #[test]
    fn test_format_multiple_panel_metrics_with_raw_tokens() {
        let usage = create_test_usage();
        let result = format_multiple_panel_metrics(
            &usage,
            &[
                PanelMetric::Cost,
                PanelMetric::InputTokens,
                PanelMetric::OutputTokens,
            ],
            true, // use_raw = true
        );
        // Should have raw token values (with possible locale separators)
        assert!(result.starts_with("$1.2 IT: "));
        // Check that the digits are preserved
        let digits_only: String = result.chars().filter(char::is_ascii_digit).collect();
        assert!(digits_only.contains("10000"));
        assert!(digits_only.contains("5000"));
    }

    #[test]
    fn test_format_multiple_panel_metrics_duplicates_handled() {
        let usage = create_test_usage();
        // Test that duplicates in input are handled (should appear only once)
        let result = format_multiple_panel_metrics(
            &usage,
            &[
                PanelMetric::Cost,
                PanelMetric::Cost,
                PanelMetric::Interactions,
            ],
            false,
        );
        assert_eq!(result, "$1.2 5x");
    }
}
