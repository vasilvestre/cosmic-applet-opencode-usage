// SPDX-License-Identifier: GPL-3.0-only

//! UI rendering logic for the viewer application.

use crate::viewer::Message;
use cosmic::{
    iced::{Alignment, Length},
    widget::{column, container, row, text},
    Element,
};

/// Renders the main content view for the viewer application.
///
/// This displays a simple test chart with sample data to verify the implementation works.
#[must_use]
pub fn view_content() -> Element<'static, Message> {
    // Sample data for testing
    let data = vec![
        ("Mon", 15000, 8000),
        ("Tue", 22000, 12000),
        ("Wed", 18000, 9500),
        ("Thu", 25000, 13000),
        ("Fri", 20000, 10500),
        ("Sat", 12000, 6000),
        ("Sun", 10000, 5500),
    ];

    let max_value = 30000;

    // Build the chart
    let mut chart_column = column()
        .push(
            text("Weekly Token Usage - Test Chart")
                .size(24)
                .width(Length::Fill),
        )
        .push(
            text("Visual bar chart representation (text-based)")
                .size(14)
                .width(Length::Fill),
        )
        .spacing(20)
        .padding(40)
        .align_x(Alignment::Center);

    // Add bars for each day
    for (day, input, output) in data {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let input_bars = ((input as f32 / max_value as f32) * 40.0) as usize;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let output_bars = ((output as f32 / max_value as f32) * 40.0) as usize;

        let input_bar = "█".repeat(input_bars);
        let output_bar = "▓".repeat(output_bars);

        let day_column = column()
            .push(text(format!("{day:>3}")).size(14))
            .push(
                row()
                    .push(text("Input:  ").size(12).width(Length::Fixed(80.0)))
                    .push(text(input_bar).size(12))
                    .push(text(format!(" {input}")).size(12))
                    .spacing(5),
            )
            .push(
                row()
                    .push(text("Output: ").size(12).width(Length::Fixed(80.0)))
                    .push(text(output_bar).size(12))
                    .push(text(format!(" {output}")).size(12))
                    .spacing(5),
            )
            .spacing(5);

        chart_column = chart_column.push(day_column);
    }

    container(chart_column)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(200.0)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_content_renders() {
        // Test that view_content returns a valid Element
        let _element = view_content();
        // If this compiles and runs, the test passes
    }
}
