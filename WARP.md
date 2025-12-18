# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Overview

clickup-tui is a Rust terminal UI application that organizes ClickUp tasks by responsibility rather than due dates. It uses ratatui for the TUI, tokio/reqwest for async API calls, and maintains local state (pins, snoozes) separately from ClickUp data.

## Commands

```bash
# Build
cargo build --release

# Run (requires config at ~/.config/clickup-tui/config.toml)
cargo run --release

# Run tests
cargo test

# Check/lint
cargo check
cargo clippy
```

## Architecture

### Module Structure

- `main.rs` - Entry point, terminal setup, event loop
- `app.rs` - Application state (`App` struct) and input handling logic
- `api.rs` - ClickUp API client (`ClickUpClient`)
- `models.rs` - Data models: `Task`, `TaskGroup`, `LocalState`, `TaskOverlay`, `DisplayTask`
- `config.rs` - Config loading/saving from TOML
- `ui.rs` - Ratatui rendering (tabs, task list, status bar)
- `theme.rs` - Spaceduck color palette constants

### Key Concepts

**Task Groups** - Tasks are categorized by responsibility status, not ClickUp's native organization:
- MyAction: Tasks you need to work on
- Waiting: Ball in someone else's court (testing, review)
- Backlog: Not yet prioritized
- Done: Completed/cancelled
- Snoozed: Locally hidden until a date

**Local Overlay** - `LocalState` stores per-task data (pins, snoozes) in `local_state.json`, separate from ClickUp. `DisplayTask` combines a `Task` with its `TaskOverlay` for rendering.

**Status Mapping** - `status_to_group()` in `models.rs` maps ClickUp status strings to `TaskGroup`. Modify this function to customize grouping logic.

### Data Flow

1. `Config::load()` reads API credentials from `~/.config/clickup-tui/config.toml`
2. `ClickUpClient::fetch_tasks()` gets tasks assigned to the configured user
3. `App::set_tasks()` stores tasks and updates local state
4. `App::current_tasks()` filters/sorts tasks for display, applying local overlays
5. `ui::render()` draws the current state to terminal

### Configuration

All files stored in `~/.config/clickup-tui/`:
- `config.toml` - API token, user_id, auto_refresh setting
- `local_state.json` - Pins, snoozes, last refresh timestamp
- `tasks_cache.json` - Cached tasks for offline viewing

## Dependencies

- **ratatui/crossterm** - TUI framework and terminal handling
- **tokio/reqwest** - Async runtime and HTTP client
- **serde/serde_json/toml** - Serialization
- **chrono** - Date/time handling for snooze logic
- **open** - Open URLs in browser
- **anyhow/thiserror** - Error handling
