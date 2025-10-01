// SPDX-License-Identifier: GPL-3.0-only

//! Static chart image generation for usage data visualization.

use crate::core::database::repository::UsageSnapshot;
use chrono::NaiveDate;
use image::{Rgba, RgbaImage};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

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

/// Generates a static token usage chart as an `RgbaImage`.
///
/// This creates a pre-rendered image that can be displayed without per-frame rendering,
/// eliminating scroll lag issues.
#[must_use]
pub fn generate_token_usage_chart(
    snapshots: &[UsageSnapshot],
    width: u32,
    height: u32,
) -> RgbaImage {
    let data = prepare_daily_tokens_data(snapshots);

    // Create pixmap for drawing
    let mut pixmap = Pixmap::new(width, height).expect("Failed to create pixmap");

    // Fill white background
    pixmap.fill(Color::WHITE);

    if data.is_empty() {
        // Just return white background with "No data" message
        // For now, we'll return the blank image
        return pixmap_to_rgba_image(&pixmap);
    }

    // Calculate chart dimensions with margins
    let margin = 40.0;
    #[allow(clippy::cast_precision_loss)]
    let chart_width = width as f32 - 2.0 * margin;
    #[allow(clippy::cast_precision_loss)]
    let chart_height = height as f32 - 2.0 * margin;

    // Find max value for scaling
    let max_tokens = data
        .iter()
        .map(|(_, input, output, reasoning)| *input.max(output).max(reasoning))
        .max()
        .unwrap_or(1000);

    #[allow(clippy::cast_precision_loss)]
    let max_tokens_f = max_tokens as f32;

    let mut paint = Paint::default();
    paint.anti_alias = true;

    // Draw axes
    paint.set_color(Color::from_rgba8(180, 180, 180, 255));
    let mut stroke = Stroke::default();
    stroke.width = 1.0;

    // X-axis
    let mut pb = PathBuilder::new();
    #[allow(clippy::cast_precision_loss)]
    pb.move_to(margin, height as f32 - margin);
    #[allow(clippy::cast_precision_loss)]
    pb.line_to(width as f32 - margin, height as f32 - margin);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    // Y-axis
    let mut pb = PathBuilder::new();
    #[allow(clippy::cast_precision_loss)]
    pb.move_to(margin, margin);
    #[allow(clippy::cast_precision_loss)]
    pb.line_to(margin, height as f32 - margin);
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    if data.len() < 2 {
        return pixmap_to_rgba_image(&pixmap);
    }

    // Draw input tokens line (blue)
    paint.set_color(Color::from_rgba8(50, 100, 200, 255));
    stroke.width = 2.0;
    let mut pb = PathBuilder::new();
    let mut first = true;

    for (i, (_, input, _, _)) in data.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let x = margin + (i as f32 / (data.len() - 1) as f32) * chart_width;
        #[allow(clippy::cast_precision_loss)]
        let y = height as f32 - margin - (*input as f32 / max_tokens_f) * chart_height;

        if first {
            pb.move_to(x, y);
            first = false;
        } else {
            pb.line_to(x, y);
        }
    }

    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    // Draw output tokens line (red)
    paint.set_color(Color::from_rgba8(200, 50, 50, 255));
    let mut pb = PathBuilder::new();
    let mut first = true;

    for (i, (_, _, output, _)) in data.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let x = margin + (i as f32 / (data.len() - 1) as f32) * chart_width;
        #[allow(clippy::cast_precision_loss)]
        let y = height as f32 - margin - (*output as f32 / max_tokens_f) * chart_height;

        if first {
            pb.move_to(x, y);
            first = false;
        } else {
            pb.line_to(x, y);
        }
    }

    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    // Draw reasoning tokens line (green)
    paint.set_color(Color::from_rgba8(50, 150, 50, 255));
    let mut pb = PathBuilder::new();
    let mut first = true;

    for (i, (_, _, _, reasoning)) in data.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let x = margin + (i as f32 / (data.len() - 1) as f32) * chart_width;
        #[allow(clippy::cast_precision_loss)]
        let y = height as f32 - margin - (*reasoning as f32 / max_tokens_f) * chart_height;

        if first {
            pb.move_to(x, y);
            first = false;
        } else {
            pb.line_to(x, y);
        }
    }

    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    pixmap_to_rgba_image(&pixmap)
}

/// Converts a `tiny_skia::Pixmap` to an `image::RgbaImage`.
fn pixmap_to_rgba_image(pixmap: &Pixmap) -> RgbaImage {
    let width = pixmap.width();
    let height = pixmap.height();
    let mut img = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = pixmap.pixel(x, y).expect("Pixel out of bounds");
            img.put_pixel(
                x,
                y,
                Rgba([pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]),
            );
        }
    }

    img
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

    #[test]
    fn test_generate_token_usage_chart_creates_image() {
        let snapshots = vec![
            create_test_snapshot(NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(), 1000, 100),
            create_test_snapshot(NaiveDate::from_ymd_opt(2025, 10, 2).unwrap(), 2000, 200),
        ];

        let img = generate_token_usage_chart(&snapshots, 800, 400);
        assert_eq!(img.width(), 800);
        assert_eq!(img.height(), 400);
    }

    #[test]
    fn test_generate_token_usage_chart_empty_data() {
        let img = generate_token_usage_chart(&[], 800, 400);
        assert_eq!(img.width(), 800);
        assert_eq!(img.height(), 400);
    }
}
