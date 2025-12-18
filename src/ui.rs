//! TUI rendering with ratatui

use crate::app::{App, FocusedPane, InputMode};
use crate::models::TaskGroup;
use crate::theme;
use crate::models::DisplayTask;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

/// Render the entire UI
pub fn render(frame: &mut Frame, app: &App) {
    // Add outer margin for breathing room
    let outer_area = frame.area().inner(Margin { horizontal: 1, vertical: 0 });

    // In search mode, show search-specific split pane
    if app.input_mode == InputMode::Search {
        render_search_mode(frame, app);
    } else {
        // Normal mode with split pane (task list + preview)
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Task list + Preview
                Constraint::Length(3), // Status bar
            ])
            .split(outer_area);

        render_tabs(frame, app, main_chunks[0]);

        // Split content area: task list (55%) | preview (45%) with gap
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(55),
                Constraint::Length(1), // Gap between panes
                Constraint::Percentage(45),
            ])
            .split(main_chunks[1]);

        render_task_list(frame, app, content_chunks[0]);
        render_normal_preview_pane(frame, app, content_chunks[2]);

        render_status_bar(frame, app, main_chunks[2]);
    }

    // Render help overlay if active
    if app.show_help {
        render_help_overlay(frame);
    }
}

/// Render help overlay with legend
fn render_help_overlay(frame: &mut Frame) {
    let area = frame.area();
    
    // Center the help popup (70% width, 80% height)
    let popup_width = (area.width * 70 / 100).min(80);
    let popup_height = (area.height * 80 / 100).min(35);
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    let help_content: Vec<Line<'static>> = vec![
        Line::from(Span::styled("KEYBINDINGS", Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  j/k, ↑/↓  ", Style::default().fg(theme::CYAN)),
            Span::styled("Navigate tasks", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  h/l, Tab  ", Style::default().fg(theme::CYAN)),
            Span::styled("Switch tabs", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  1-6       ", Style::default().fg(theme::CYAN)),
            Span::styled("Jump to tab (My Action, Waiting, Backlog, Done, Snoozed, Person)", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  o, Enter  ", Style::default().fg(theme::CYAN)),
            Span::styled("Open task in browser", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  y         ", Style::default().fg(theme::CYAN)),
            Span::styled("Copy task to clipboard", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  p         ", Style::default().fg(theme::CYAN)),
            Span::styled("Toggle pin", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  s         ", Style::default().fg(theme::CYAN)),
            Span::styled("Snooze task", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  S         ", Style::default().fg(theme::CYAN)),
            Span::styled("Unsnooze task", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  /         ", Style::default().fg(theme::CYAN)),
            Span::styled("Global fuzzy search", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  r         ", Style::default().fg(theme::CYAN)),
            Span::styled("Refresh tasks from ClickUp", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  ?         ", Style::default().fg(theme::CYAN)),
            Span::styled("Toggle this help", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  q         ", Style::default().fg(theme::CYAN)),
            Span::styled("Quit", Style::default().fg(theme::FG)),
        ]),
        Line::from(""),
        Line::from(Span::styled("PRIORITY INDICATORS", Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  !!  ", Style::default().fg(theme::ORANGE)),
            Span::styled("Urgent", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  !   ", Style::default().fg(theme::PURPLE)),
            Span::styled("High", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  -   ", Style::default().fg(theme::YELLOW)),
            Span::styled("Normal", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  ·   ", Style::default().fg(theme::MUTED)),
            Span::styled("Low", Style::default().fg(theme::FG)),
        ]),
        Line::from(""),
        Line::from(Span::styled("SYMBOLS", Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📌  ", Style::default().fg(theme::YELLOW)),
            Span::styled("Pinned task", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  └   ", Style::default().fg(theme::MUTED)),
            Span::styled("Subtask (child of another task)", Style::default().fg(theme::FG)),
        ]),
        Line::from(""),
        Line::from(Span::styled("STATUS COLORS", Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ████  ", Style::default().fg(theme::STATUS_IN_PROGRESS)),
            Span::styled("In Progress", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  ████  ", Style::default().fg(theme::STATUS_TODO)),
            Span::styled("To Do", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  ████  ", Style::default().fg(theme::STATUS_BLOCKED)),
            Span::styled("Blocked", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  ████  ", Style::default().fg(theme::STATUS_TESTING)),
            Span::styled("In Testing", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  ████  ", Style::default().fg(theme::STATUS_VALIDATE)),
            Span::styled("To Validate", Style::default().fg(theme::FG)),
        ]),
        Line::from(vec![
            Span::styled("  ████  ", Style::default().fg(theme::STATUS_DONE)),
            Span::styled("Done / Completed", Style::default().fg(theme::FG)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Press Esc, q, or ? to close", Style::default().fg(theme::MUTED))),
    ];

    let help = Paragraph::new(help_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::BLUE))
                .title(Span::styled(
                    " Help ",
                    Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().bg(theme::SELECTED_BG));

    frame.render_widget(help, popup_area);
}

/// Render preview pane for selected task in normal mode
fn render_normal_preview_pane(frame: &mut Frame, app: &App, area: Rect) {
    let selected = app.selected_task();

    let content: Vec<Line> = if let Some(dt) = selected {
        build_preview_content(&dt, area.width as usize)
    } else {
        vec![Line::from(Span::styled(
            "No task selected",
            Style::default().fg(theme::MUTED),
        ))]
    };

    let border_color = if app.focused_pane == FocusedPane::Preview {
        theme::CYAN
    } else {
        theme::MUTED
    };

    let preview = Paragraph::new(content)
        .wrap(Wrap { trim: true })
        .scroll((app.preview_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    " Details ",
                    Style::default().fg(theme::CYAN),
                )),
        );

    frame.render_widget(preview, area);
}

/// Render search mode with split pane (results left, preview right)
fn render_search_mode(frame: &mut Frame, app: &App) {
    // Add outer margin for breathing room
    let outer_area = frame.area().inner(Margin { horizontal: 1, vertical: 0 });

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),    // Results + Preview
            Constraint::Length(3), // Status bar
        ])
        .split(outer_area);

    // Search input bar
    let search_input = Paragraph::new(Line::from(vec![
        Span::styled(" 🔍 ", Style::default().fg(theme::BLUE)),
        Span::styled(&app.search_query, Style::default().fg(theme::FG)),
        Span::styled("│", Style::default().fg(theme::BLUE)), // cursor
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::BLUE))
            .title(Span::styled(
                " Global Search ",
                Style::default()
                    .fg(theme::BLUE)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(search_input, main_chunks[0]);

    // Split middle area: results (55%) | gap | preview (45%)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Length(1), // Gap between panes
            Constraint::Percentage(45),
        ])
        .split(main_chunks[1]);

    // Render search results
    render_search_results(frame, app, content_chunks[0]);

    // Render preview pane
    render_preview_pane(frame, app, content_chunks[2]);

    // Status bar
    render_status_bar(frame, app, main_chunks[2]);
}

/// Render search results list
fn render_search_results(frame: &mut Frame, app: &App, area: Rect) {
    let results = app.search_all_tasks();

    let items: Vec<ListItem> = results
        .iter()
        .enumerate()
        .map(|(idx, dt)| {
            let is_selected = idx == app.search_selected_index;

            // Priority indicator
            let priority_style = match dt.task.priority {
                Some(1) => Style::default().fg(theme::ORANGE),
                Some(2) => Style::default().fg(theme::PURPLE),
                Some(3) => Style::default().fg(theme::YELLOW),
                _ => Style::default().fg(theme::MUTED),
            };
            let priority_indicator = match dt.task.priority {
                Some(1) => "!! ",
                Some(2) => "!  ",
                Some(3) => "-  ",
                _ => "   ",
            };

            let status_style = get_status_style(&dt.task.status);

            // Truncate name
            let max_len = area.width.saturating_sub(20) as usize;
            let name = if dt.task.name.len() > max_len {
                format!("{}...", &dt.task.name[..max_len.saturating_sub(3)])
            } else {
                dt.task.name.clone()
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(theme::FG)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::FG)
            };

            let line = Line::from(vec![
                Span::styled(priority_indicator, priority_style),
                Span::styled(name, name_style),
                Span::raw("  "),
                Span::styled(&dt.task.status, status_style),
            ]);

            let item = ListItem::new(line);
            if is_selected {
                item.style(Style::default().bg(theme::SELECTED_BG))
            } else {
                item
            }
        })
        .collect();

    let title = if results.is_empty() {
        if app.search_query.is_empty() {
            " Type to search... ".to_string()
        } else {
            " No matches ".to_string()
        }
    } else {
        format!(" {} results ", results.len())
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::MUTED))
            .title(Span::styled(title, Style::default().fg(theme::FG))),
    );

    frame.render_widget(list, area);
}

/// Render preview pane for selected search result
fn render_preview_pane(frame: &mut Frame, app: &App, area: Rect) {
    let selected = app.selected_search_result();

    let content: Vec<Line> = if let Some(dt) = selected {
        build_preview_content(&dt, area.width as usize)
    } else {
        vec![Line::from(Span::styled(
            "No task selected",
            Style::default().fg(theme::MUTED),
        ))]
    };

    let preview = Paragraph::new(content)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::MUTED))
                .title(Span::styled(
                    " Preview ",
                    Style::default().fg(theme::CYAN),
                )),
        );

    frame.render_widget(preview, area);
}

/// Build preview content for a task (returns owned Lines)
fn build_preview_content(dt: &DisplayTask, _width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Custom ID if present (e.g., "PROJ-123")
    if let Some(custom_id) = &dt.task.custom_id {
        lines.push(Line::from(Span::styled(
            custom_id.clone(),
            Style::default().fg(theme::CYAN).add_modifier(Modifier::BOLD),
        )));
    }

    // Task name (bold)
    lines.push(Line::from(Span::styled(
        dt.task.name.clone(),
        Style::default()
            .fg(theme::FG)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Task type
    if let Some(task_type) = dt.task.task_type_label() {
        lines.push(Line::from(vec![
            Span::styled("Type: ", Style::default().fg(theme::MUTED)),
            Span::styled(task_type, Style::default().fg(theme::PINK)),
        ]));
    }

    // Subtask indicator
    if dt.task.is_subtask() {
        lines.push(Line::from(vec![
            Span::styled("└ ", Style::default().fg(theme::MUTED)),
            Span::styled("Subtask", Style::default().fg(theme::MUTED)),
        ]));
    }

    // Status
    let status_style = get_status_style(&dt.task.status);
    lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(theme::MUTED)),
        Span::styled(dt.task.status.clone(), status_style),
    ]));

    // List
    lines.push(Line::from(vec![
        Span::styled("List: ", Style::default().fg(theme::MUTED)),
        Span::styled(dt.task.list_name.clone(), Style::default().fg(theme::FG)),
    ]));

    // Priority
    if let Some(p) = dt.task.priority_label() {
        let priority_style = match dt.task.priority {
            Some(1) => Style::default().fg(theme::ORANGE),
            Some(2) => Style::default().fg(theme::PURPLE),
            Some(3) => Style::default().fg(theme::YELLOW),
            _ => Style::default().fg(theme::MUTED),
        };
        lines.push(Line::from(vec![
            Span::styled("Priority: ", Style::default().fg(theme::MUTED)),
            Span::styled(p, priority_style),
        ]));
    }

    // Tags
    if !dt.task.tags.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Tags: ", Style::default().fg(theme::MUTED)),
            Span::styled(dt.task.tags.join(", "), Style::default().fg(theme::CYAN)),
        ]));
    }

    // Pin status
    if dt.overlay.pinned {
        lines.push(Line::from(Span::styled(
            "📌 Pinned",
            Style::default().fg(theme::YELLOW),
        )));
    }

    // Description
    if let Some(desc) = &dt.task.description {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Description:",
            Style::default()
                .fg(theme::MUTED)
                .add_modifier(Modifier::BOLD),
        )));
        // Show full description (scrollable)
        for line in desc.lines() {
            lines.push(Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(theme::FG),
            )));
        }
    }

    lines
}

/// Render the tab bar
fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let counts = app.group_counts();

    let titles: Vec<Line> = TaskGroup::all()
        .iter()
        .map(|&group| {
            let count = counts
                .iter()
                .find(|(g, _)| *g == group)
                .map(|(_, c)| *c)
                .unwrap_or(0);

            let style = if group == app.current_group {
                Style::default()
                    .fg(theme::TAB_ACTIVE)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TAB_INACTIVE)
            };

            Line::from(vec![
                Span::styled(format!("{} ", group.label()), style),
                Span::styled(format!("({})", count), Style::default().fg(theme::MUTED)),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::MUTED))
                .title(Span::styled(
                    " ClickUp Tasks ",
                    Style::default()
                        .fg(theme::BLUE)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .select(app.current_group.index())
        .style(Style::default().fg(theme::FG))
        .highlight_style(
            Style::default()
                .fg(theme::TAB_ACTIVE)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(" │ ", Style::default().fg(theme::MUTED)));

    frame.render_widget(tabs, area);
}

/// Get status style color
fn get_status_style(status: &str) -> Style {
    match status.to_lowercase().as_str() {
        "in progress" => Style::default().fg(theme::STATUS_IN_PROGRESS),
        "to do" | "todo" | "to-do" => Style::default().fg(theme::STATUS_TODO),
        "to review" | "in review" | "review" => Style::default().fg(theme::STATUS_IN_PROGRESS), // Actionable like in progress
        "blocked" => Style::default().fg(theme::STATUS_BLOCKED),
        "in testing" | "testing" => Style::default().fg(theme::STATUS_TESTING),
        "to validate" | "validation" => Style::default().fg(theme::STATUS_VALIDATE),
        "backlog" => Style::default().fg(theme::STATUS_BACKLOG),
        "done" | "completed" | "released" => Style::default().fg(theme::STATUS_DONE),
        "cancelled" | "canceled" => Style::default().fg(theme::STATUS_CANCELLED),
        _ => Style::default().fg(theme::FG), // Default to normal text, not muted
    }
}

/// Render the task list (no status sections, status shown inline)
fn render_task_list(frame: &mut Frame, app: &App, area: Rect) {
    let tasks = app.current_tasks();

    // Build set of task IDs in view for subtask detection
    let visible_ids: std::collections::HashSet<String> = tasks.iter().map(|dt| dt.task.id.clone()).collect();
    
    // Build map of task_id -> all_tasks for depth calculation
    let all_tasks_map: std::collections::HashMap<String, &DisplayTask> = 
        tasks.iter().map(|dt| (dt.task.id.clone(), dt)).collect();

    let mut items: Vec<ListItem> = Vec::new();
    let mut task_index = 0;

    for dt in &tasks {
        let is_selected = task_index == app.selected_index;
        
        // Calculate depth (how many ancestors are visible)
        let mut depth = 0usize;
        let mut current_parent = dt.task.parent_id.clone();
        while let Some(pid) = current_parent {
            if visible_ids.contains(&pid) {
                depth += 1;
                current_parent = all_tasks_map.get(&pid).and_then(|p| p.task.parent_id.clone());
            } else {
                break;
            }
        }

        // Check if user is assigned to this task
        let is_assigned = app.user_id
            .map(|uid| dt.task.is_assigned_to(uid))
            .unwrap_or(true);

        // Pin indicator (2 chars)
        let pin_icon = if dt.overlay.pinned { "📌" } else { "  " };

        // Priority indicator (2 chars)
        let priority_style = match dt.task.priority {
            Some(1) => Style::default().fg(theme::ORANGE),
            Some(2) => Style::default().fg(theme::PURPLE),
            Some(3) => Style::default().fg(theme::YELLOW),
            _ => Style::default().fg(theme::MUTED),
        };
        let priority_indicator = match dt.task.priority {
            Some(1) => "!!",
            Some(2) => "! ",
            Some(3) => "- ",
            Some(4) => "· ",
            _ => "  ",
        };

        // Status tag - gray out if not assigned
        let status_style = if is_assigned {
            get_status_style(&dt.task.status)
        } else {
            Style::default().fg(theme::MUTED)
        };
        let status_tag = format!("[{}] ", dt.task.status);

        // Task type tag
        let type_tag = dt.task.task_type_label()
            .map(|t| format!("[{}] ", t))
            .unwrap_or_default();

        // Custom ID
        let custom_id_str = dt.task.custom_id.as_ref()
            .map(|id| format!("{} ", id))
            .unwrap_or_default();

        // Name styling - gray out unassigned tasks
        let name_style = if !is_assigned {
            Style::default().fg(theme::MUTED)
        } else if is_selected {
            Style::default().fg(theme::FG).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::FG)
        };

        // Build spans - all tasks start with pin+priority (4 chars), subtasks add indent after
        let mut spans: Vec<Span> = Vec::new();
        
        spans.push(Span::raw(pin_icon));
        spans.push(Span::styled(priority_indicator, priority_style));
        spans.push(Span::raw(" ")); // spacing
        
        // Add depth-based indentation for nested tasks
        if depth > 0 {
            // Add spaces for each level of depth, then the tree character
            let indent = "  ".repeat(depth.saturating_sub(1));
            spans.push(Span::styled(format!("{}└ ", indent), Style::default().fg(theme::MUTED)));
        }

        // Status inline
        spans.push(Span::styled(status_tag, status_style));

        // Type tag
        if !type_tag.is_empty() {
            spans.push(Span::styled(type_tag, Style::default().fg(theme::PINK)));
        }

        // Custom ID with spacing
        if !custom_id_str.is_empty() {
            spans.push(Span::styled(custom_id_str, Style::default().fg(theme::CYAN)));
        }

        // Task name
        spans.push(Span::styled(dt.task.name.clone(), name_style));

        let line = Line::from(spans);
        let item = if is_selected {
            ListItem::new(line).style(Style::default().bg(theme::SELECTED_BG))
        } else {
            ListItem::new(line)
        };
        items.push(item);

        task_index += 1;
    }

    let title = if app.input_mode == InputMode::Search {
        format!(" Search: {} ", app.search_query)
    } else if tasks.is_empty() {
        " No tasks ".to_string()
    } else {
        format!(" {} tasks ", tasks.len())
    };

    let border_color = if app.focused_pane == FocusedPane::TaskList {
        theme::CYAN
    } else {
        theme::MUTED
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(theme::FG))),
    );

    frame.render_widget(list, area);
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let content = match app.input_mode {
        InputMode::Normal => {
            if let Some(msg) = &app.status_message {
                Line::from(vec![Span::styled(msg, Style::default().fg(theme::GREEN))])
            } else if app.is_loading {
                Line::from(vec![Span::styled(
                    "Loading...",
                    Style::default().fg(theme::YELLOW),
                )])
            } else {
                // Keybinding hints
                Line::from(vec![
                    Span::styled("[j/k]", Style::default().fg(theme::BLUE)),
                    Span::styled(" nav ", Style::default().fg(theme::MUTED)),
                    Span::styled("[h/l]", Style::default().fg(theme::BLUE)),
                    Span::styled(" tabs ", Style::default().fg(theme::MUTED)),
                    Span::styled("[p]", Style::default().fg(theme::BLUE)),
                    Span::styled("in ", Style::default().fg(theme::MUTED)),
                    Span::styled("[s]", Style::default().fg(theme::BLUE)),
                    Span::styled("nooze ", Style::default().fg(theme::MUTED)),
                    Span::styled("[o]", Style::default().fg(theme::BLUE)),
                    Span::styled("pen ", Style::default().fg(theme::MUTED)),
                    Span::styled("[y]", Style::default().fg(theme::BLUE)),
                    Span::styled("ank ", Style::default().fg(theme::MUTED)),
                    Span::styled("[/]", Style::default().fg(theme::BLUE)),
                    Span::styled("search ", Style::default().fg(theme::MUTED)),
                    Span::styled("[r]", Style::default().fg(theme::BLUE)),
                    Span::styled("efresh ", Style::default().fg(theme::MUTED)),
                    Span::styled("[?]", Style::default().fg(theme::BLUE)),
                    Span::styled("help ", Style::default().fg(theme::MUTED)),
                    Span::styled("[q]", Style::default().fg(theme::BLUE)),
                    Span::styled("uit", Style::default().fg(theme::MUTED)),
                ])
            }
        }
        InputMode::Search => Line::from(vec![
            Span::styled("[j/k]", Style::default().fg(theme::BLUE)),
            Span::styled(" select ", Style::default().fg(theme::MUTED)),
            Span::styled("[Enter]", Style::default().fg(theme::BLUE)),
            Span::styled(" open ", Style::default().fg(theme::MUTED)),
            Span::styled("[Esc]", Style::default().fg(theme::BLUE)),
            Span::styled(" cancel", Style::default().fg(theme::MUTED)),
        ]),
        InputMode::Snooze => Line::from(vec![
            Span::styled("Days: ", Style::default().fg(theme::MUTED)),
            Span::styled(&app.snooze_input, Style::default().fg(theme::FG)),
            Span::styled(" ", Style::default()),
            Span::styled("[Esc]", Style::default().fg(theme::BLUE)),
            Span::styled(" cancel, ", Style::default().fg(theme::MUTED)),
            Span::styled("[Enter]", Style::default().fg(theme::BLUE)),
            Span::styled(" confirm", Style::default().fg(theme::MUTED)),
        ]),
        InputMode::Help => Line::from(vec![
            Span::styled("[Esc/q/?]", Style::default().fg(theme::BLUE)),
            Span::styled(" close help", Style::default().fg(theme::MUTED)),
        ]),
    };

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::MUTED)),
    );

    frame.render_widget(paragraph, area);
}
