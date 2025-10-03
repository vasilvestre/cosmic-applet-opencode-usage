// SPDX-License-Identifier: GPL-3.0-only

//! UI rendering logic for the viewer application.

use crate::core::database::repository::WeekSummary;
use crate::viewer::Message;
use ::image::RgbaImage;
use chrono::NaiveDate;
use cosmic::{
    iced::{Alignment, Length},
    iced_core::image::Handle,
    widget::{column, container, image as cosmic_image, row, text},
    Element,
};

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

/// Renders a metric block with current value, change indicator, and previous value.
fn metric_block(
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
    let comparison = format!("(Last: {previous_str})");

    column()
        .push(text(label_text).size(14))
        .push(text(value_display).size(18))
        .push(text(comparison).size(12))
        .spacing(4)
        .padding(12)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .into()
}

/// Renders a metric block for cost with current value, change indicator, and previous value.
fn cost_metric_block(
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
    let comparison = format!("(Last: {previous_str})");

    column()
        .push(text(label_text).size(14))
        .push(text(value_display).size(18))
        .push(text(comparison).size(12))
        .spacing(4)
        .padding(12)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .into()
}

/// Helper to render the pre-generated chart image.
fn render_chart_image(chart_image: &RgbaImage) -> Element<'static, Message> {
    let width = chart_image.width();
    let height = chart_image.height();
    let pixels = chart_image.as_raw().clone();

    let handle = Handle::from_rgba(width, height, pixels);

    container(cosmic_image(handle))
        .width(Length::Shrink)
        .height(Length::Shrink)
        .center_x(Length::Fill)
        .into()
}

/// Renders the main content view for the viewer application.
///
/// Displays week-over-week comparison in a 5-column horizontal layout,
/// with a static pre-rendered chart below.
#[must_use]
pub fn view_content(
    this_week: Option<WeekSummary>,
    last_week: Option<WeekSummary>,
    week_starts: (NaiveDate, NaiveDate),
    chart_image: &RgbaImage,
) -> Element<'_, Message> {
    let (_this_week_start, last_week_start) = week_starts;

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
            // Show all metrics in a horizontal row layout
            let metrics_row = row()
                .push(metric_block(
                    "Input",
                    "📝",
                    tw.total_input_tokens,
                    lw.total_input_tokens,
                ))
                .push(metric_block(
                    "Output",
                    "📤",
                    tw.total_output_tokens,
                    lw.total_output_tokens,
                ))
                .push(metric_block(
                    "Reasoning",
                    "🧠",
                    tw.total_reasoning_tokens,
                    lw.total_reasoning_tokens,
                ))
                .push(cost_metric_block(
                    "Cost",
                    "💰",
                    tw.total_cost,
                    lw.total_cost,
                ))
                .push(metric_block(
                    "Interactions",
                    "🔄",
                    tw.total_interactions,
                    lw.total_interactions,
                ))
                .spacing(10)
                .width(Length::Fill);

            content = content.push(metrics_row);
        }
        (Some(tw), None) => {
            // Only this week data - reuse metric block helpers with 0 for previous values
            content = content
                .push(text("No data available for last week").size(14))
                .push(text("").size(10));

            let metrics_row = row()
                .push(metric_block("Input", "📝", tw.total_input_tokens, 0))
                .push(metric_block("Output", "📤", tw.total_output_tokens, 0))
                .push(metric_block(
                    "Reasoning",
                    "🧠",
                    tw.total_reasoning_tokens,
                    0,
                ))
                .push(cost_metric_block("Cost", "💰", tw.total_cost, 0.0))
                .push(metric_block("Interactions", "🔄", tw.total_interactions, 0))
                .spacing(10)
                .width(Length::Fill);

            content = content.push(metrics_row);
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

    // Add the pre-rendered static chart
    content = content
        .push(text("").size(20)) // Spacer
        .push(text("30-Day History").size(20))
        .push(render_chart_image(chart_image));

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1_000_000), "1,000,000");
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
