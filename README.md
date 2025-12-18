# clickup-tui

A terminal UI for organizing ClickUp tasks with a personal workflow overlay.

![Spaceduck themed TUI](https://img.shields.io/badge/theme-spaceduck-purple)

## The Problem

ClickUp's "My Work" view is broken for developer workflows:
- Sorts by due dates (most are null or inaccurate)
- Doesn't reflect actual responsibility ("in testing" = QA owns it, but still assigned to you)
- No personal organization layer - team structure != personal workflow

## The Solution

This TUI fetches your ClickUp tasks and organizes them by **responsibility**, not due dates:

| Group | What It Means |
|-------|---------------|
| **My Action** | Tasks you need to work on (in progress, to-do, blocked) |
| **Waiting** | Ball is in someone else's court (in testing, to validate) |
| **Backlog** | Not yet prioritized |
| **Done** | Completed, cancelled, or for reference |
| **Snoozed** | Tasks you've hidden until a specific date |
| **Person** | Long-standing role/person type tasks |

Plus a **personal overlay** that persists locally:
- **Pin** important tasks to the top
- **Snooze** tasks you can't deal with right now
- **Search** across all tasks

## Installation

### From Releases

Download the latest binary for your platform from [Releases](https://github.com/Randallsm83/clickup-tui/releases).

### From Source

```bash
git clone https://github.com/Randallsm83/clickup-tui
cd clickup-tui
cargo build --release

# Binary is at target/release/clickup-tui
# Or install globally:
cargo install --path .
```

## Configuration

On first run, the app creates a config file at `~/.config/clickup-tui/config.toml` (all platforms).

Edit the config file with your ClickUp credentials:

```toml
# Get your API token from: ClickUp Settings > Apps > API Token
api_token = "pk_YOUR_API_TOKEN_HERE"

# Your ClickUp user ID (numeric)
# Find it in ClickUp URL when viewing your profile, or use the MCP server
user_id = "12345678"

# Auto-refresh on startup (default: true)
auto_refresh = true
```

### Finding Your User ID

If you have the ClickUp MCP server configured:
1. Ask: "What's my ClickUp user ID?"
2. Or check the workspace members list

Alternatively, look at the network tab in ClickUp's web app when loading your tasks.

## Usage

```bash
clickup-tui
```

### Keybindings

| Key | Action |
|-----|--------|
| `j/k` or arrows | Navigate tasks |
| `h/l` | Switch tabs |
| `1-6` | Jump to tab (My Action, Waiting, Backlog, Done, Snoozed, Person) |
| `Tab` | Switch pane focus |
| `p` | Toggle pin on selected task |
| `s` | Snooze task (enter days) |
| `S` | Unsnooze task |
| `o` or `Enter` | Open task in browser |
| `y` | Copy task to clipboard |
| `r` | Refresh from ClickUp |
| `/` | Global fuzzy search |
| `?` | Show help |
| `q` | Quit |

## Data Storage

All data is stored locally in `~/.config/clickup-tui/`:
- `config.toml` - API token and settings
- `local_state.json` - Pins, snoozes, custom ordering
- `tasks_cache.json` - Cached tasks for offline viewing

No data is ever sent anywhere except to ClickUp's API.

## Theme

Uses the [Spaceduck](https://github.com/pineapplegiant/spaceduck) color palette by default.

## License

MIT
