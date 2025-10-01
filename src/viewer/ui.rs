// SPDX-License-Identifier: GPL-3.0-only

//! UI rendering logic for the viewer application.

use crate::viewer::Message;
use cosmic::{
    iced::{Alignment, Length},
    widget::{container, text},
    Element,
};

/// Renders the main content view for the viewer application.
///
/// This is currently a placeholder that will be replaced with actual
/// historical data display in a future feature.
#[must_use]
pub fn view_content() -> Element<'static, Message> {
    container(
        text("Historical data will be displayed here")
            .size(16)
            .width(Length::Fill)
            .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(200.0)
    .center_y(200.0)
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
