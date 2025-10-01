// SPDX-License-Identifier: GPL-3.0-only

//! Viewer application module for displaying historical `OpenCode` usage data.
//!
//! This module provides a standalone COSMIC application that shares database
//! infrastructure with the applet for viewing historical usage data.

pub mod app;
pub mod ui;

pub use app::{Message, ViewerApp};
