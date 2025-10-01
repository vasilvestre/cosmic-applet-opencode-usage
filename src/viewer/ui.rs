// SPDX-License-Identifier: GPL-3.0-only

//! UI rendering logic for the viewer application.

use crate::core::database::repository::UsageRepository;
use crate::viewer::charts;
use crate::viewer::Message;
use chrono::{Datelike, NaiveDate, Utc};
use cosmic::{
    iced::{Alignment, Length},
    widget::{column, container, text},
    Element,
};
use std::sync::Arc;

/// Calculates the start of the week (Monday) for a given date.
fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(i64::from(weekday))
}

/// Formats a number with thousands separators.
fn format_number(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Formats a cost value.
fn format_cost(cost: f64) -> String {
    format!("${cost:.2}")
}

/// Calculates percentage change and returns formatted string with arrow.
fn format_change(current: i64, previous: i64) -> (String, String) {
    if previous == 0 {
        if current == 0 {
            return ("0%".to_string(), "-".to_string());
        }
        return ("+INF%".to_string(), "UP".to_string());
    }

    #[allow(clippy::cast_precision_loss)]
    let change_pct = ((current - previous) as f64 / previous as f64) * 100.0;

    let arrow = if change_pct > 0.0 {
        "UP"
    } else if change_pct < 0.0 {
        "DN"
    } else {
        "--"
    };

    (format!("{change_pct:+.1}%"), arrow.to_string())
}

/// Calculates percentage change for costs.
fn format_cost_change(current: f64, previous: f64) -> (String, String) {
    if previous == 0.0 {
        if current == 0.0 {
            return ("0%".to_string(), "-".to_string());
        }
        return ("+INF%".to_string(), "UP".to_string());
    }

    let change_pct = ((current - previous) / previous) * 100.0;

    let arrow = if change_pct > 0.0 {
        "UP"
    } else if change_pct < 0.0 {
        "DN"
    } else {
        "--"
    };

    (format!("{change_pct:+.1}%"), arrow.to_string())
}

/// Renders a comparison row for a metric.
#[allow(clippy::too_many_arguments)]
fn comparison_row(
    label: &str,
    icon: &str,
    current: i64,
    previous: i64,
) -> cosmic::Element<'static, Message> {
    let (change_text, arrow) = format_change(current, previous);
    let current_str = format_number(current);
    let previous_str = format_number(previous);

    let label_text = format!("{icon} {label}");
    let value_display = format!("{current_str}  {arrow}  {change_text}");
    let comparison = format!("(Last week: {previous_str})");

    column()
        .push(text(label_text).size(16))
        .push(text(value_display).size(20))
        .push(text(comparison).size(13))
        .spacing(4)
        .padding([12, 0])
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .into()
}

/// Renders a comparison row for cost metric.
fn cost_comparison_row(
    label: &str,
    icon: &str,
    current: f64,
    previous: f64,
) -> cosmic::Element<'static, Message> {
    let (change_text, arrow) = format_cost_change(current, previous);
    let current_str = format_cost(current);
    let previous_str = format_cost(previous);

    let label_text = format!("{icon} {label}");
    let value_display = format!("{current_str}  {arrow}  {change_text}");
    let comparison = format!("(Last week: {previous_str})");

    column()
        .push(text(label_text).size(16))
        .push(text(value_display).size(20))
        .push(text(comparison).size(13))
        .spacing(4)
        .padding([12, 0])
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .into()
}

/// Renders the main content view for the viewer application.
///
/// Displays week-over-week comparison of usage metrics.
#[must_use]
pub fn view_content(repository: &Arc<UsageRepository>) -> Element<'static, Message> {
    let today = Utc::now().date_naive();
    let this_week_start = get_week_start(today);
    let last_week_start = this_week_start - chrono::Duration::days(7);

    // Fetch data
    let this_week = match repository.get_week_summary(this_week_start) {
        Ok(summary) => Some(summary),
        Err(e) => {
            eprintln!("ERROR: Failed to get this week summary: {e}");
            None
        }
    };

    let last_week = match repository.get_week_summary(last_week_start) {
        Ok(summary) => Some(summary),
        Err(e) => {
            eprintln!("ERROR: Failed to get last week summary: {e}");
            None
        }
    };

    let mut content = column()
        .push(
            text("OpenCode Usage - Weekly Comparison")
                .size(28)
                .width(Length::Fill),
        )
        .spacing(20)
        .padding(40)
        .align_x(Alignment::Center);

    // Add date range info
    if let Some(ref tw) = this_week {
        content = content.push(
            text(format!(
                "This Week: {} - {} vs Last Week: {} - {}",
                tw.start_date.format("%b %d"),
                tw.end_date.format("%b %d"),
                last_week_start.format("%b %d"),
                (last_week_start + chrono::Duration::days(6)).format("%b %d")
            ))
            .size(14),
        );
    }

    content = content.push(text("").size(10)); // Spacer

    match (this_week, last_week) {
        (Some(tw), Some(lw)) => {
            // Show all comparisons
            content = content
                .push(comparison_row(
                    "Input Tokens",
                    "📝",
                    tw.total_input_tokens,
                    lw.total_input_tokens,
                ))
                .push(comparison_row(
                    "Output Tokens",
                    "📤",
                    tw.total_output_tokens,
                    lw.total_output_tokens,
                ))
                .push(comparison_row(
                    "Reasoning Tokens",
                    "🧠",
                    tw.total_reasoning_tokens,
                    lw.total_reasoning_tokens,
                ))
                .push(cost_comparison_row(
                    "Total Cost",
                    "💰",
                    tw.total_cost,
                    lw.total_cost,
                ))
                .push(comparison_row(
                    "Interactions",
                    "🔄",
                    tw.total_interactions,
                    lw.total_interactions,
                ));
        }
        (Some(tw), None) => {
            // Only this week data
            content = content
                .push(text("No data available for last week").size(14))
                .push(text("").size(10))
                .push(
                    column()
                        .push(text("📝 Input Tokens").size(16))
                        .push(text(format_number(tw.total_input_tokens)).size(20))
                        .spacing(4)
                        .padding([12, 0])
                        .align_x(Alignment::Center)
                        .width(Length::Fill),
                )
                .push(
                    column()
                        .push(text("📤 Output Tokens").size(16))
                        .push(text(format_number(tw.total_output_tokens)).size(20))
                        .spacing(4)
                        .padding([12, 0])
                        .align_x(Alignment::Center)
                        .width(Length::Fill),
                )
                .push(
                    column()
                        .push(text("🧠 Reasoning Tokens").size(16))
                        .push(text(format_number(tw.total_reasoning_tokens)).size(20))
                        .spacing(4)
                        .padding([12, 0])
                        .align_x(Alignment::Center)
                        .width(Length::Fill),
                )
                .push(
                    column()
                        .push(text("💰 Total Cost").size(16))
                        .push(text(format_cost(tw.total_cost)).size(20))
                        .spacing(4)
                        .padding([12, 0])
                        .align_x(Alignment::Center)
                        .width(Length::Fill),
                )
                .push(
                    column()
                        .push(text("🔄 Interactions").size(16))
                        .push(text(format_number(tw.total_interactions)).size(20))
                        .spacing(4)
                        .padding([12, 0])
                        .align_x(Alignment::Center)
                        .width(Length::Fill),
                );
        }
        (None, Some(_lw)) => {
            content = content.push(text("No data available for this week yet").size(14));
        }
        (None, None) => {
            content = content
                .push(text("No usage data available").size(16))
                .push(text("").size(10))
                .push(text("Start using OpenCode to see statistics here!").size(14));
        }
    }

    // Add historical chart for last 30 days
    let end_date = today;
    let start_date = today - chrono::Duration::days(30);
    
    match repository.get_range(start_date, end_date) {
        Ok(snapshots) if !snapshots.is_empty() => {
            content = content
                .push(text("").size(20)) // Spacer
                .push(text("30-Day History").size(20))
                .push(charts::token_usage_chart(&snapshots));
        }
        Ok(_) => {
            // No data for chart range
        }
        Err(e) => {
            eprintln!("ERROR: Failed to fetch chart data: {e}");
        }
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(200.0)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::database::DatabaseManager;
    use tempfile::TempDir;

    #[test]
    fn test_view_content_renders_with_empty_data() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Arc::new(DatabaseManager::new_with_path(&db_path).unwrap());
        let repository = Arc::new(UsageRepository::new(db));

        // Test that view_content returns a valid Element with empty database
        let _element = view_content(&repository);
        // If this compiles and runs, the test passes
    }

    #[test]
    fn test_get_week_start_monday() {
        // Monday Oct 27, 2025
        let monday = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();
        assert_eq!(get_week_start(monday), monday);
    }

    #[test]
    fn test_get_week_start_friday() {
        // Friday Oct 31, 2025 -> should return Monday Oct 27
        let friday = NaiveDate::from_ymd_opt(2025, 10, 31).unwrap();
        let expected = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();
        assert_eq!(get_week_start(friday), expected);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1000000), "1,000,000");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_format_change() {
        let (pct, arrow) = format_change(120, 100);
        assert_eq!(arrow, "UP");
        assert!(pct.contains("20"));

        let (pct, arrow) = format_change(80, 100);
        assert_eq!(arrow, "DN");
        assert!(pct.contains("-20"));

        let (pct, arrow) = format_change(100, 100);
        assert_eq!(arrow, "--");
        assert_eq!(pct, "+0.0%");
    }
}
