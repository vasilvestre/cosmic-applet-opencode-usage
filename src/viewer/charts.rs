// SPDX-License-Identifier: GPL-3.0-only

//! Chart rendering for usage data visualization.

use crate::core::database::repository::UsageSnapshot;
use crate::viewer::Message;
use chrono::NaiveDate;
use cosmic::iced::mouse;
use cosmic::iced::widget::canvas::{self, Geometry, Path, Stroke, Text};
use cosmic::iced::widget::Canvas;
use cosmic::iced::{Point, Rectangle};
use cosmic::{Renderer, Theme};
use cosmic::iced::{Color, Length};
use cosmic::Element;

/// Prepares daily token usage data for charting.
///
/// Returns a vector of (date, input_tokens, output_tokens, reasoning_tokens) tuples
/// sorted by date ascending.
#[must_use]
pub fn prepare_daily_tokens_data(
    snapshots: &[UsageSnapshot],
) -> Vec<(NaiveDate, i64, i64, i64)> {
    let mut data: Vec<_> = snapshots
        .iter()
        .map(|s| {
            (
                s.date,
                s.input_tokens,
                s.output_tokens,
                s.reasoning_tokens,
            )
        })
        .collect();

    data.sort_by_key(|(date, _, _, _)| *date);
    data
}

/// Prepares daily cost data for charting.
///
/// Returns a vector of (date, cost) tuples sorted by date ascending.
#[must_use]
pub fn prepare_daily_cost_data(snapshots: &[UsageSnapshot]) -> Vec<(NaiveDate, f64)> {
    let mut data: Vec<_> = snapshots.iter().map(|s| (s.date, s.total_cost)).collect();

    data.sort_by_key(|(date, _)| *date);
    data
}

/// Prepares daily interaction count data for charting.
///
/// Returns a vector of (date, interactions) tuples sorted by date ascending.
#[must_use]
pub fn prepare_daily_interactions_data(snapshots: &[UsageSnapshot]) -> Vec<(NaiveDate, i64)> {
    let mut data: Vec<_> = snapshots
        .iter()
        .map(|s| (s.date, s.interaction_count))
        .collect();

    data.sort_by_key(|(date, _)| *date);
    data
}

/// Line chart showing token usage over time.
#[derive(Debug)]
pub struct TokenUsageChart {
    data: Vec<(NaiveDate, i64, i64, i64)>,
    cache: canvas::Cache,
}

impl TokenUsageChart {
    /// Creates a new token usage chart from snapshots.
    #[must_use]
    pub fn new(snapshots: &[UsageSnapshot]) -> Self {
        Self {
            data: prepare_daily_tokens_data(snapshots),
            cache: canvas::Cache::default(),
        }
    }
}

impl canvas::Program<Message, Theme, Renderer> for TokenUsageChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            if self.data.is_empty() {
                let text = Text {
                    content: "No data available".to_string(),
                    position: Point::new(bounds.width / 2.0, bounds.height / 2.0),
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    size: 16.0.into(),
                    ..Text::default()
                };
                frame.fill_text(text);
                return;
            }

            // Calculate chart dimensions with margins
            let margin = 40.0;
            let chart_width = bounds.width - 2.0 * margin;
            let chart_height = bounds.height - 2.0 * margin;

            // Find max value for scaling
            let max_tokens = self
                .data
                .iter()
                .map(|(_, input, output, reasoning)| *input.max(output).max(reasoning))
                .max()
                .unwrap_or(1000);

            #[allow(clippy::cast_precision_loss)]
            let max_tokens_f = max_tokens as f32;

            // Draw axes
            let x_axis = Path::line(
                Point::new(margin, bounds.height - margin),
                Point::new(bounds.width - margin, bounds.height - margin),
            );
            let y_axis = Path::line(
                Point::new(margin, margin),
                Point::new(margin, bounds.height - margin),
            );

            frame.stroke(
                &x_axis,
                Stroke::default().with_color(Color::from_rgb(0.7, 0.7, 0.7)),
            );
            frame.stroke(
                &y_axis,
                Stroke::default().with_color(Color::from_rgb(0.7, 0.7, 0.7)),
            );

            // Draw title
            let title = Text {
                content: "Token Usage Over Time".to_string(),
                position: Point::new(bounds.width / 2.0, 15.0),
                color: Color::from_rgb(0.2, 0.2, 0.2),
                size: 18.0.into(),
                ..Text::default()
            };
            frame.fill_text(title);

            if self.data.len() < 2 {
                return;
            }

            // Draw input tokens line (blue)
            let input_points: Vec<Point> = self
                .data
                .iter()
                .enumerate()
                .map(|(i, (_, input, _, _))| {
                    #[allow(clippy::cast_precision_loss)]
                    let x = margin + (i as f32 / (self.data.len() - 1) as f32) * chart_width;
                    #[allow(clippy::cast_precision_loss)]
                    let y = bounds.height
                        - margin
                        - (*input as f32 / max_tokens_f) * chart_height;
                    Point::new(x, y)
                })
                .collect();

            if input_points.len() >= 2 {
                for i in 0..input_points.len() - 1 {
                    let line = Path::line(input_points[i], input_points[i + 1]);
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_width(2.0)
                            .with_color(Color::from_rgb(0.2, 0.4, 0.8)),
                    );
                }
            }

            // Draw output tokens line (red)
            let output_points: Vec<Point> = self
                .data
                .iter()
                .enumerate()
                .map(|(i, (_, _, output, _))| {
                    #[allow(clippy::cast_precision_loss)]
                    let x = margin + (i as f32 / (self.data.len() - 1) as f32) * chart_width;
                    #[allow(clippy::cast_precision_loss)]
                    let y = bounds.height
                        - margin
                        - (*output as f32 / max_tokens_f) * chart_height;
                    Point::new(x, y)
                })
                .collect();

            if output_points.len() >= 2 {
                for i in 0..output_points.len() - 1 {
                    let line = Path::line(output_points[i], output_points[i + 1]);
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_width(2.0)
                            .with_color(Color::from_rgb(0.8, 0.2, 0.2)),
                    );
                }
            }

            // Draw legend
            let legend_y = margin + 20.0;
            let input_legend = Path::line(
                Point::new(bounds.width - 150.0, legend_y),
                Point::new(bounds.width - 130.0, legend_y),
            );
            frame.stroke(
                &input_legend,
                Stroke::default()
                    .with_width(2.0)
                    .with_color(Color::from_rgb(0.2, 0.4, 0.8)),
            );
            let input_text = Text {
                content: "Input".to_string(),
                position: Point::new(bounds.width - 125.0, legend_y - 8.0),
                color: Color::from_rgb(0.2, 0.4, 0.8),
                size: 12.0.into(),
                ..Text::default()
            };
            frame.fill_text(input_text);

            let output_legend = Path::line(
                Point::new(bounds.width - 150.0, legend_y + 20.0),
                Point::new(bounds.width - 130.0, legend_y + 20.0),
            );
            frame.stroke(
                &output_legend,
                Stroke::default()
                    .with_width(2.0)
                    .with_color(Color::from_rgb(0.8, 0.2, 0.2)),
            );
            let output_text = Text {
                content: "Output".to_string(),
                position: Point::new(bounds.width - 125.0, legend_y + 12.0),
                color: Color::from_rgb(0.8, 0.2, 0.2),
                size: 12.0.into(),
                ..Text::default()
            };
            frame.fill_text(output_text);
        });

        vec![geometry]
    }
}

/// Creates a token usage chart widget.
#[must_use]
pub fn token_usage_chart<'a>(snapshots: Vec<UsageSnapshot>) -> Element<'a, Message> {
    let chart = TokenUsageChart::new(&snapshots);
    Canvas::new(chart)
        .width(Length::Fill)
        .height(Length::Fixed(300.0))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_test_snapshot(date: NaiveDate, input: i64, output: i64) -> UsageSnapshot {
        UsageSnapshot {
            date,
            input_tokens: input,
            output_tokens: output,
            reasoning_tokens: 100,
            cache_write_tokens: 0,
            cache_read_tokens: 0,
            total_cost: 1.5,
            interaction_count: 10,
        }
    }

    #[test]
    fn test_prepare_daily_tokens_data_empty() {
        let data = prepare_daily_tokens_data(&[]);
        assert!(data.is_empty());
    }

    #[test]
    fn test_prepare_daily_tokens_data_sorts_by_date() {
        let snapshots = vec![
            create_test_snapshot(NaiveDate::from_ymd_opt(2025, 10, 3).unwrap(), 3000, 300),
            create_test_snapshot(NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(), 1000, 100),
            create_test_snapshot(NaiveDate::from_ymd_opt(2025, 10, 2).unwrap(), 2000, 200),
        ];

        let data = prepare_daily_tokens_data(&snapshots);

        assert_eq!(data.len(), 3);
        assert_eq!(data[0].0, NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
        assert_eq!(data[1].0, NaiveDate::from_ymd_opt(2025, 10, 2).unwrap());
        assert_eq!(data[2].0, NaiveDate::from_ymd_opt(2025, 10, 3).unwrap());
    }

    #[test]
    fn test_prepare_daily_tokens_data_extracts_correct_values() {
        let snapshots = vec![create_test_snapshot(
            NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
            5000,
            500,
        )];

        let data = prepare_daily_tokens_data(&snapshots);

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].1, 5000); // input_tokens
        assert_eq!(data[0].2, 500); // output_tokens
        assert_eq!(data[0].3, 100); // reasoning_tokens
    }

    #[test]
    fn test_prepare_daily_cost_data_sorts_by_date() {
        let snapshots = vec![
            create_test_snapshot(NaiveDate::from_ymd_opt(2025, 10, 3).unwrap(), 3000, 300),
            create_test_snapshot(NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(), 1000, 100),
        ];

        let data = prepare_daily_cost_data(&snapshots);

        assert_eq!(data.len(), 2);
        assert_eq!(data[0].0, NaiveDate::from_ymd_opt(2025, 10, 1).unwrap());
        assert_eq!(data[1].0, NaiveDate::from_ymd_opt(2025, 10, 3).unwrap());
    }

    #[test]
    fn test_prepare_daily_interactions_data() {
        let snapshots = vec![create_test_snapshot(
            NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
            1000,
            100,
        )];

        let data = prepare_daily_interactions_data(&snapshots);

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].1, 10); // interaction_count
    }
}
