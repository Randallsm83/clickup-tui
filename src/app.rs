//! TUI application state and logic

use crate::config::Config;
use crate::models::{DisplayTask, LocalState, Task, TaskGroup};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use std::fs;

/// Input mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
    Snooze,
    Help,
}

/// Which pane has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPane {
    #[default]
    TaskList,
    Preview,
}

/// Application state
pub struct App {
    /// All tasks from ClickUp
    pub tasks: Vec<Task>,
    /// Local state (pins, snoozes)
    pub local_state: LocalState,
    /// Current tab/group
    pub current_group: TaskGroup,
    /// Selected task index within current group
    pub selected_index: usize,
    /// Search/filter query
    pub search_query: String,
    /// Current input mode
    pub input_mode: InputMode,
    /// Snooze input buffer
    pub snooze_input: String,
    /// Status message to display
    pub status_message: Option<String>,
    /// Whether app should quit
    pub should_quit: bool,
    /// Whether data is loading
    pub is_loading: bool,
    /// Selected index in global search results
    pub search_selected_index: usize,
    /// Show help screen
    pub show_help: bool,
    /// Current user's ID (for checking task assignment)
    pub user_id: Option<u64>,
    /// Which pane is focused
    pub focused_pane: FocusedPane,
    /// Preview pane scroll offset
    pub preview_scroll: u16,
}

impl App {
    /// Create a new app instance
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            local_state: LocalState::default(),
            current_group: TaskGroup::MyAction,
            selected_index: 0,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            snooze_input: String::new(),
            status_message: None,
            should_quit: false,
            is_loading: false,
            search_selected_index: 0,
            show_help: false,
            user_id: None,
            focused_pane: FocusedPane::TaskList,
            preview_scroll: 0,
        }
    }

    /// Move focus to next pane (Ctrl+l)
    pub fn focus_next_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::TaskList => FocusedPane::Preview,
            FocusedPane::Preview => FocusedPane::TaskList,
        };
    }

    /// Move focus to previous pane (Ctrl+h)
    pub fn focus_prev_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::TaskList => FocusedPane::Preview,
            FocusedPane::Preview => FocusedPane::TaskList,
        };
    }

    /// Scroll preview down
    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    /// Scroll preview up
    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    /// Reset preview scroll when task changes
    pub fn reset_preview_scroll(&mut self) {
        self.preview_scroll = 0;
    }

    /// Set the user ID from config
    pub fn set_user_id(&mut self, user_id: &str) {
        self.user_id = user_id.parse().ok();
    }

    /// Load local state from disk
    pub fn load_local_state(&mut self) -> Result<()> {
        let path = Config::state_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read state from {}", path.display()))?;
            self.local_state = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse state from {}", path.display()))?;
        }
        Ok(())
    }

    /// Save local state to disk
    pub fn save_local_state(&self) -> Result<()> {
        let path = Config::state_path()?;
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir)?;
        let content = serde_json::to_string_pretty(&self.local_state)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Load cached tasks from disk
    pub fn load_cached_tasks(&mut self) -> Result<()> {
        let path = Config::cache_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            self.tasks = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    /// Save tasks to cache
    pub fn save_tasks_cache(&self) -> Result<()> {
        let path = Config::cache_path()?;
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir)?;
        let content = serde_json::to_string_pretty(&self.tasks)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Set tasks and update local state timestamp
    pub fn set_tasks(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
        self.local_state.last_refresh = Some(Utc::now());
        self.selected_index = 0;
    }

    /// Get display tasks for the current group
    pub fn current_tasks(&self) -> Vec<DisplayTask> {
        use std::collections::{HashMap, HashSet};

        let user_id = self.user_id;

        // Build all display tasks indexed by ID
        let all_tasks: HashMap<String, DisplayTask> = self
            .tasks
            .iter()
            .map(|t| {
                (
                    t.id.clone(),
                    DisplayTask::new(t.clone(), self.local_state.get_overlay(&t.id)),
                )
            })
            .collect();

        // Get tasks assigned to user in this group (iterate self.tasks for stable order)
        let my_tasks: Vec<DisplayTask> = self
            .tasks
            .iter()
            .map(|t| DisplayTask::new(t.clone(), self.local_state.get_overlay(&t.id)))
            .filter(|dt| {
                let in_group = if self.current_group == TaskGroup::Person {
                    dt.task.custom_item_id == Some(1020)
                } else {
                    dt.task.custom_item_id != Some(1020)
                        && dt.effective_group() == self.current_group
                };
                let is_assigned = user_id
                    .map(|uid| dt.task.is_assigned_to(uid))
                    .unwrap_or(true);
                in_group && is_assigned
            })
            .filter(|dt| {
                if self.search_query.is_empty() {
                    true
                } else {
                    let query = self.search_query.to_lowercase();
                    dt.task.name.to_lowercase().contains(&query)
                        || dt.task.list_name.to_lowercase().contains(&query)
                        || dt.task.status.to_lowercase().contains(&query)
                        || dt
                            .task
                            .description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&query))
                            .unwrap_or(false)
                }
            })
            .collect();

        // Build set of tasks to include (my tasks + their ancestors)
        let mut included: Vec<DisplayTask> = Vec::new();
        let mut added_ids: HashSet<String> = HashSet::new();

        for dt in &my_tasks {
            // Add ancestor chain (stop at first unassigned ancestor)
            let mut ancestors: Vec<DisplayTask> = Vec::new();
            let mut current_parent_id = dt.task.parent_id.clone();

            while let Some(pid) = current_parent_id {
                if let Some(parent) = all_tasks.get(&pid) {
                    ancestors.push(parent.clone());
                    let parent_assigned = user_id
                        .map(|uid| parent.task.is_assigned_to(uid))
                        .unwrap_or(true);
                    if !parent_assigned {
                        break;
                    }
                    current_parent_id = parent.task.parent_id.clone();
                } else {
                    break;
                }
            }

            // Add ancestors
            for ancestor in ancestors.into_iter().rev() {
                if !added_ids.contains(&ancestor.task.id) {
                    included.push(ancestor.clone());
                    added_ids.insert(ancestor.task.id.clone());
                }
            }

            // Add the task itself
            if !added_ids.contains(&dt.task.id) {
                included.push(dt.clone());
                added_ids.insert(dt.task.id.clone());
            }
        }

        // Build a map of id -> depth (for sorting)
        let mut depth_map: HashMap<String, usize> = HashMap::new();
        for dt in &included {
            let mut depth = 0;
            let mut pid = dt.task.parent_id.clone();
            while let Some(p) = pid {
                if added_ids.contains(&p) {
                    depth += 1;
                    pid = all_tasks.get(&p).and_then(|t| t.task.parent_id.clone());
                } else {
                    break;
                }
            }
            depth_map.insert(dt.task.id.clone(), depth);
        }

        // Helper to get root ancestor within visible set
        let get_root = |id: &str, parent_id: &Option<String>| -> String {
            let mut root = id.to_string();
            let mut current = parent_id.clone();
            while let Some(pid) = current {
                if added_ids.contains(&pid) {
                    root = pid.clone();
                    current = all_tasks.get(&pid).and_then(|t| t.task.parent_id.clone());
                } else {
                    break;
                }
            }
            root
        };

        // Sort: root tasks by priority, then children under their parents
        included.sort_by(|a, b| {
            let root_a = get_root(&a.task.id, &a.task.parent_id);
            let root_b = get_root(&b.task.id, &b.task.parent_id);

            // Compare by root's priority
            let root_a_priority = all_tasks.get(&root_a).and_then(|t| t.task.priority);
            let root_b_priority = all_tasks.get(&root_b).and_then(|t| t.task.priority);

            let priority_cmp = match (root_a_priority, root_b_priority) {
                (Some(pa), Some(pb)) => pa.cmp(&pb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            };
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }

            // Group by root
            if root_a != root_b {
                return root_a.cmp(&root_b);
            }

            // Same family: sort by depth (parents before children)
            let depth_a = depth_map.get(&a.task.id).unwrap_or(&0);
            let depth_b = depth_map.get(&b.task.id).unwrap_or(&0);
            let depth_cmp = depth_a.cmp(depth_b);
            if depth_cmp != std::cmp::Ordering::Equal {
                return depth_cmp;
            }

            // Final tiebreaker: task ID for stable sort
            a.task.id.cmp(&b.task.id)
        });

        included
    }

    /// Get count of tasks in each group
    pub fn group_counts(&self) -> Vec<(TaskGroup, usize)> {
        TaskGroup::all()
            .iter()
            .map(|&group| {
                let count = self
                    .tasks
                    .iter()
                    .map(|t| DisplayTask::new(t.clone(), self.local_state.get_overlay(&t.id)))
                    .filter(|dt| {
                        if group == TaskGroup::Person {
                            dt.task.custom_item_id == Some(1020)
                        } else {
                            dt.task.custom_item_id != Some(1020) && dt.effective_group() == group
                        }
                    })
                    .count();
                (group, count)
            })
            .collect()
    }

    /// Get currently selected task
    pub fn selected_task(&self) -> Option<DisplayTask> {
        let tasks = self.current_tasks();
        tasks.get(self.selected_index).cloned()
    }

    /// Search all tasks globally (across all groups) with fuzzy matching
    pub fn search_all_tasks(&self) -> Vec<DisplayTask> {
        if self.search_query.is_empty() {
            return Vec::new();
        }

        let query = self.search_query.to_lowercase();
        let query_chars: Vec<char> = query.chars().collect();

        let mut results: Vec<(DisplayTask, i32)> = self
            .tasks
            .iter()
            .map(|t| DisplayTask::new(t.clone(), self.local_state.get_overlay(&t.id)))
            .filter_map(|dt| {
                let score = fuzzy_score(&dt.task.name, &query_chars)
                    .or_else(|| fuzzy_score(&dt.task.list_name, &query_chars))
                    .or_else(|| fuzzy_score(&dt.task.status, &query_chars))
                    .or_else(|| {
                        dt.task
                            .description
                            .as_ref()
                            .and_then(|d| fuzzy_score(d, &query_chars))
                    })
                    .or_else(|| {
                        dt.task
                            .tags
                            .iter()
                            .find_map(|tag| fuzzy_score(tag, &query_chars))
                    });
                score.map(|s| (dt, s))
            })
            .collect();

        // Sort by score (higher is better)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter().map(|(dt, _)| dt).collect()
    }

    /// Get currently selected search result
    pub fn selected_search_result(&self) -> Option<DisplayTask> {
        let results = self.search_all_tasks();
        results.get(self.search_selected_index).cloned()
    }

    /// Move search selection up
    pub fn search_select_prev(&mut self) {
        if self.search_selected_index > 0 {
            self.search_selected_index -= 1;
        }
    }

    /// Move search selection down
    pub fn search_select_next(&mut self) {
        let results = self.search_all_tasks();
        if self.search_selected_index < results.len().saturating_sub(1) {
            self.search_selected_index += 1;
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let tasks = self.current_tasks();
        if self.selected_index < tasks.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Switch to a tab/group
    pub fn switch_group(&mut self, group: TaskGroup) {
        self.current_group = group;
        self.selected_index = 0;
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        let idx = (self.current_group.index() + 1) % TaskGroup::all().len();
        if let Some(group) = TaskGroup::from_index(idx) {
            self.switch_group(group);
        }
    }

    /// Switch to previous tab
    pub fn prev_tab(&mut self) {
        let len = TaskGroup::all().len();
        let idx = (self.current_group.index() + len - 1) % len;
        if let Some(group) = TaskGroup::from_index(idx) {
            self.switch_group(group);
        }
    }

    /// Toggle pin on selected task
    pub fn toggle_pin(&mut self) {
        if let Some(task) = self.selected_task() {
            self.local_state.toggle_pin(&task.task.id);
            let pinned = self.local_state.is_pinned(&task.task.id);
            self.status_message = Some(if pinned {
                "Task pinned".to_string()
            } else {
                "Task unpinned".to_string()
            });
            let _ = self.save_local_state();
        }
    }

    /// Start snooze input mode
    pub fn start_snooze(&mut self) {
        if self.selected_task().is_some() {
            self.input_mode = InputMode::Snooze;
            self.snooze_input.clear();
            self.status_message = Some("Snooze for how many days? (Enter number)".to_string());
        }
    }

    /// Confirm snooze with entered days
    pub fn confirm_snooze(&mut self) {
        if let Ok(days) = self.snooze_input.parse::<i64>() {
            if let Some(task) = self.selected_task() {
                let until = Utc::now() + Duration::days(days);
                self.local_state.snooze(&task.task.id, until);
                self.status_message = Some(format!("Task snoozed for {} days", days));
                let _ = self.save_local_state();
            }
        } else {
            self.status_message = Some("Invalid number".to_string());
        }
        self.input_mode = InputMode::Normal;
        self.snooze_input.clear();
    }

    /// Unsnooze selected task
    pub fn unsnooze(&mut self) {
        if let Some(task) = self.selected_task() {
            self.local_state.unsnooze(&task.task.id);
            self.status_message = Some("Task unsnoozed".to_string());
            let _ = self.save_local_state();
        }
    }

    /// Open selected task in browser
    pub fn open_in_browser(&mut self) {
        if let Some(task) = self.selected_task() {
            if let Err(e) = open::that(&task.task.url) {
                self.status_message = Some(format!("Failed to open: {}", e));
            } else {
                self.status_message = Some("Opened in browser".to_string());
            }
        }
    }

    /// Copy selected task name to clipboard
    pub fn copy_to_clipboard(&mut self) {
        if let Some(task) = self.selected_task() {
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    if let Err(e) = clipboard.set_text(&task.task.name) {
                        self.status_message = Some(format!("Failed to copy: {}", e));
                    } else {
                        self.status_message = Some("Copied task name".to_string());
                    }
                }
                Err(e) => {
                    self.status_message = Some(format!("Clipboard error: {}", e));
                }
            }
        }
    }

    /// Start search mode
    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_query.clear();
        self.search_selected_index = 0;
    }

    /// Exit search/snooze mode
    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_query.clear();
        self.snooze_input.clear();
    }

    /// Handle character input based on mode
    pub fn handle_char(&mut self, c: char) {
        match self.input_mode {
            InputMode::Search => {
                self.search_query.push(c);
                self.search_selected_index = 0;
            }
            InputMode::Snooze => {
                if c.is_ascii_digit() {
                    self.snooze_input.push(c);
                }
            }
            InputMode::Normal | InputMode::Help => {}
        }
    }

    /// Handle backspace in input modes
    pub fn handle_backspace(&mut self) {
        match self.input_mode {
            InputMode::Search => {
                self.search_query.pop();
                self.search_selected_index = 0;
            }
            InputMode::Snooze => {
                self.snooze_input.pop();
            }
            InputMode::Normal | InputMode::Help => {}
        }
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple fuzzy matching score - returns Some(score) if all query chars found in order
fn fuzzy_score(text: &str, query_chars: &[char]) -> Option<i32> {
    if query_chars.is_empty() {
        return Some(0);
    }

    let text_lower = text.to_lowercase();
    let text_chars: Vec<char> = text_lower.chars().collect();

    let mut query_idx = 0;
    let mut score = 0i32;
    let mut last_match_idx: Option<usize> = None;
    let mut consecutive_bonus = 0;

    for (text_idx, &tc) in text_chars.iter().enumerate() {
        if query_idx < query_chars.len() && tc == query_chars[query_idx] {
            // Bonus for consecutive matches
            if let Some(last) = last_match_idx {
                if text_idx == last + 1 {
                    consecutive_bonus += 5;
                }
            }

            // Bonus for matching at word boundaries
            if text_idx == 0 || !text_chars[text_idx - 1].is_alphanumeric() {
                score += 10;
            }

            score += 1;
            last_match_idx = Some(text_idx);
            query_idx += 1;
        }
    }

    if query_idx == query_chars.len() {
        Some(score + consecutive_bonus)
    } else {
        None
    }
}
