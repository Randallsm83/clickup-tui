//! Spaceduck theme colors for the TUI
//!
//! Based on https://github.com/pineapplegiant/spaceduck

use ratatui::style::Color;

// Spaceduck palette
pub const FG: Color = Color::Rgb(236, 240, 193); // #ecf0c1
pub const PURPLE: Color = Color::Rgb(242, 206, 0); // #f2ce00 (ANSI magenta)
pub const PINK: Color = Color::Rgb(206, 111, 143); // #ce6f8f
pub const GREEN: Color = Color::Rgb(92, 204, 150); // #5ccc96
pub const ORANGE: Color = Color::Rgb(227, 52, 0); // #e33400 (ANSI red)
pub const BLUE: Color = Color::Rgb(0, 163, 204); // #00a3cc
pub const CYAN: Color = Color::Rgb(122, 92, 204); // #7a5ccc
pub const YELLOW: Color = Color::Rgb(179, 161, 230); // #b3a1e6

// Semantic colors
pub const SELECTED_BG: Color = Color::Rgb(30, 34, 54); // Slightly lighter bg
pub const MUTED: Color = Color::Rgb(100, 100, 120);

// Status colors
pub const STATUS_IN_PROGRESS: Color = BLUE;
pub const STATUS_TODO: Color = YELLOW;
pub const STATUS_BLOCKED: Color = ORANGE;
pub const STATUS_TESTING: Color = CYAN;
pub const STATUS_VALIDATE: Color = PINK;
pub const STATUS_BACKLOG: Color = MUTED;
pub const STATUS_DONE: Color = GREEN;
pub const STATUS_CANCELLED: Color = MUTED;

// Tab colors
pub const TAB_ACTIVE: Color = BLUE;
pub const TAB_INACTIVE: Color = MUTED;
