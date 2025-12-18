//! Terminal user interface for soltop
//!
//! This module contains the TUI app that displays network statistics
//! in an interactive terminal dashboard.

mod app;
mod theme;

pub use app::App;
pub use theme::Theme;