//! Data models for tasks and local state

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task group based on responsibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TaskGroup {
    #[default]
    MyAction,
    Waiting,
    Backlog,
    Done,
    Snoozed,
    /// Long-standing role/person type tasks (custom_item_id = 1020)
    Person,
}

impl TaskGroup {
    pub fn all() -> &'static [TaskGroup] {
        &[
            TaskGroup::MyAction,
            TaskGroup::Waiting,
            TaskGroup::Backlog,
            TaskGroup::Done,
            TaskGroup::Snoozed,
            TaskGroup::Person,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TaskGroup::MyAction => "My Action",
            TaskGroup::Waiting => "Waiting",
            TaskGroup::Backlog => "Backlog",
            TaskGroup::Done => "Done",
            TaskGroup::Snoozed => "Snoozed",
            TaskGroup::Person => "Person",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            TaskGroup::MyAction => 0,
            TaskGroup::Waiting => 1,
            TaskGroup::Backlog => 2,
            TaskGroup::Done => 3,
            TaskGroup::Snoozed => 4,
            TaskGroup::Person => 5,
        }
    }

    pub fn from_index(idx: usize) -> Option<TaskGroup> {
        match idx {
            0 => Some(TaskGroup::MyAction),
            1 => Some(TaskGroup::Waiting),
            2 => Some(TaskGroup::Backlog),
            3 => Some(TaskGroup::Done),
            4 => Some(TaskGroup::Snoozed),
            5 => Some(TaskGroup::Person),
            _ => None,
        }
    }
}

/// Map ClickUp status to task group
pub fn status_to_group(status: &str) -> TaskGroup {
    let status_lower = status.to_lowercase();
    match status_lower.as_str() {
        // My Action - I need to do something
        "in progress" | "to do" | "to-do" | "todo" => TaskGroup::MyAction,
        "in review" | "review" | "to review" => TaskGroup::MyAction, // Reviews are actionable

        // Waiting - Ball in someone else's court
        "blocked" => TaskGroup::Waiting, // Can't act until unblocked
        "in testing" | "testing" => TaskGroup::Waiting,
        "to validate" | "validation" | "pending review" => TaskGroup::Waiting,

        // Backlog - Not yet prioritized
        "backlog" | "open" | "new" => TaskGroup::Backlog,

        // Done - Completed
        "done" | "complete" | "completed" | "closed" => TaskGroup::Done,
        "released" | "deployed" | "shipped" => TaskGroup::Done,
        "cancelled" | "canceled" | "won't do" | "wontdo" => TaskGroup::Done,
        "for reference" => TaskGroup::Done,

        // Default to backlog for unknown statuses
        _ => TaskGroup::Backlog,
    }
}

/// A task from ClickUp with local overlay data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// ClickUp task ID
    pub id: String,
    /// Task name/title
    pub name: String,
    /// ClickUp status
    pub status: String,
    /// List name the task belongs to
    pub list_name: String,
    /// Due date (Unix timestamp in ms)
    pub due_date: Option<i64>,
    /// Priority (1=Urgent, 2=High, 3=Normal, 4=Low)
    pub priority: Option<u8>,
    /// URL to open in browser
    pub url: String,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
    /// Task description/content
    #[serde(default)]
    pub description: Option<String>,
    /// Custom task type ID (e.g., 1020 = Back-End Developer/Person)
    #[serde(default)]
    pub custom_item_id: Option<u32>,
    /// Custom task ID (e.g., "PROJ-123")
    #[serde(default)]
    pub custom_id: Option<String>,
    /// Parent task ID (if this is a subtask)
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Assignee user IDs
    #[serde(default)]
    pub assignee_ids: Vec<u64>,
}

impl Task {
    /// Get the task group based on status
    pub fn group(&self) -> TaskGroup {
        status_to_group(&self.status)
    }

    /// Get priority label
    pub fn priority_label(&self) -> Option<&'static str> {
        match self.priority {
            Some(1) => Some("Urgent"),
            Some(2) => Some("High"),
            Some(3) => Some("Normal"),
            Some(4) => Some("Low"),
            _ => None,
        }
    }

    /// Check if this task is a subtask
    pub fn is_subtask(&self) -> bool {
        self.parent_id.is_some()
    }

    /// Check if a user is assigned to this task
    pub fn is_assigned_to(&self, user_id: u64) -> bool {
        self.assignee_ids.contains(&user_id)
    }

    /// Get task type label based on custom_item_id
    pub fn task_type_label(&self) -> Option<&'static str> {
        match self.custom_item_id {
            Some(0) => Some("Task"),
            Some(1004) => Some("Bug"),
            Some(1005) => Some("Milestone"),
            Some(1006) => Some("Feature"),
            Some(1007) => Some("Epic"),
            Some(1008) => Some("Story"),
            Some(1009) => Some("Spike"),
            Some(1020) => Some("Person"),
            Some(_) => Some("Custom"),
            None => None,
        }
    }
}

/// Local task overlay data (persisted separately from ClickUp data)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskOverlay {
    /// Whether task is pinned
    #[serde(default)]
    pub pinned: bool,
    /// Snoozed until this date
    pub snoozed_until: Option<DateTime<Utc>>,
    /// Custom sort order within group
    pub sort_order: Option<u32>,
}

/// Local state for all tasks
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalState {
    /// Overlay data keyed by task ID
    #[serde(default)]
    pub overlays: HashMap<String, TaskOverlay>,
    /// Last refresh timestamp
    pub last_refresh: Option<DateTime<Utc>>,
}

impl LocalState {
    /// Get overlay for a task, or default
    pub fn get_overlay(&self, task_id: &str) -> TaskOverlay {
        self.overlays.get(task_id).cloned().unwrap_or_default()
    }

    /// Toggle pin for a task
    pub fn toggle_pin(&mut self, task_id: &str) {
        let overlay = self.overlays.entry(task_id.to_string()).or_default();
        overlay.pinned = !overlay.pinned;
    }

    /// Snooze a task until a date
    pub fn snooze(&mut self, task_id: &str, until: DateTime<Utc>) {
        let overlay = self.overlays.entry(task_id.to_string()).or_default();
        overlay.snoozed_until = Some(until);
    }

    /// Unsnooze a task
    pub fn unsnooze(&mut self, task_id: &str) {
        if let Some(overlay) = self.overlays.get_mut(task_id) {
            overlay.snoozed_until = None;
        }
    }

    /// Check if a task is pinned
    pub fn is_pinned(&self, task_id: &str) -> bool {
        self.overlays
            .get(task_id)
            .map(|o| o.pinned)
            .unwrap_or(false)
    }
}

/// Combined task with overlay for display
#[derive(Debug, Clone)]
pub struct DisplayTask {
    pub task: Task,
    pub overlay: TaskOverlay,
}

impl DisplayTask {
    pub fn new(task: Task, overlay: TaskOverlay) -> Self {
        Self { task, overlay }
    }

    /// Determine the effective group (considering snooze)
    pub fn effective_group(&self) -> TaskGroup {
        if self
            .overlay
            .snoozed_until
            .map(|u| u > Utc::now())
            .unwrap_or(false)
        {
            TaskGroup::Snoozed
        } else {
            self.task.group()
        }
    }
}
