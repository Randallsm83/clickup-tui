//! ClickUp API client for fetching tasks

use crate::models::Task;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

const CLICKUP_API_BASE: &str = "https://api.clickup.com/api/v2";

/// ClickUp API client
pub struct ClickUpClient {
    client: Client,
    api_token: String,
}

/// Response from ClickUp task search
#[derive(Debug, Deserialize)]
struct TasksResponse {
    tasks: Vec<ClickUpTask>,
}

/// Raw task from ClickUp API
#[derive(Debug, Deserialize)]
struct ClickUpTask {
    id: String,
    name: String,
    status: ClickUpStatus,
    list: ClickUpList,
    due_date: Option<String>,
    priority: Option<ClickUpPriority>,
    url: String,
    #[serde(default)]
    tags: Vec<ClickUpTag>,
    /// Task description/content
    text_content: Option<String>,
    /// Custom task type ID (e.g., 1020 = Back-End Developer)
    custom_item_id: Option<u32>,
    /// Custom task ID (e.g., "PROJ-123")
    custom_id: Option<String>,
    /// Parent task ID (if this is a subtask)
    parent: Option<String>,
    /// Assignees
    #[serde(default)]
    assignees: Vec<ClickUpAssignee>,
}

#[derive(Debug, Deserialize)]
struct ClickUpStatus {
    status: String,
}

#[derive(Debug, Deserialize)]
struct ClickUpList {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ClickUpPriority {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ClickUpTag {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ClickUpAssignee {
    id: u64,
}

/// Response from team endpoint
#[derive(Debug, Deserialize)]
struct TeamsResponse {
    teams: Vec<Team>,
}

#[derive(Debug, Deserialize)]
struct Team {
    id: String,
    #[allow(dead_code)]
    name: String,
}

impl ClickUpClient {
    /// Create a new ClickUp client
    pub fn new(api_token: String) -> Self {
        Self {
            client: Client::new(),
            api_token,
        }
    }

    /// Get the team/workspace ID (needed for task queries)
    pub async fn get_team_id(&self) -> Result<String> {
        let url = format!("{}/team", CLICKUP_API_BASE);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.api_token)
            .send()
            .await
            .context("Failed to fetch teams")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("ClickUp API error ({}): {}", status, body);
        }

        let teams: TeamsResponse = response
            .json()
            .await
            .context("Failed to parse teams response")?;

        teams
            .teams
            .first()
            .map(|t| t.id.clone())
            .context("No teams found in workspace")
    }

    /// Fetch all tasks assigned to a user, including parent tasks of subtasks
    pub async fn fetch_tasks(&self, team_id: &str, user_id: &str) -> Result<Vec<Task>> {
        use std::collections::HashSet;

        let url = format!("{}/team/{}/task", CLICKUP_API_BASE, team_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.api_token)
            .query(&[
                ("assignees[]", user_id),
                ("include_closed", "true"),
                ("subtasks", "true"),
            ])
            .send()
            .await
            .context("Failed to fetch tasks")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("ClickUp API error ({}): {}", status, body);
        }

        let tasks_response: TasksResponse = response
            .json()
            .await
            .context("Failed to parse tasks response")?;

        let mut tasks: Vec<Task> = tasks_response
            .tasks
            .into_iter()
            .map(|t| self.convert_task(t))
            .collect();

        // Collect IDs of tasks we already have
        let existing_ids: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();

        // Find parent IDs that we don't have yet
        let missing_parent_ids: Vec<String> = tasks
            .iter()
            .filter_map(|t| t.parent_id.as_ref())
            .filter(|pid| !existing_ids.contains(*pid))
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Fetch missing parent tasks
        for parent_id in missing_parent_ids {
            if let Ok(parent_task) = self.fetch_task_by_id(&parent_id).await {
                tasks.push(parent_task);
            }
        }

        Ok(tasks)
    }

    /// Fetch a single task by ID
    pub async fn fetch_task_by_id(&self, task_id: &str) -> Result<Task> {
        let url = format!("{}/task/{}", CLICKUP_API_BASE, task_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.api_token)
            .send()
            .await
            .context("Failed to fetch task")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("ClickUp API error ({}): {}", status, body);
        }

        let task: ClickUpTask = response
            .json()
            .await
            .context("Failed to parse task response")?;

        Ok(self.convert_task(task))
    }

    /// Convert ClickUpTask to Task
    fn convert_task(&self, t: ClickUpTask) -> Task {
        Task {
            id: t.id,
            name: t.name,
            status: t.status.status,
            list_name: t.list.name,
            due_date: t.due_date.and_then(|d| d.parse().ok()),
            priority: t.priority.and_then(|p| p.id.parse().ok()),
            url: t.url,
            tags: t.tags.into_iter().map(|t| t.name).collect(),
            description: t.text_content,
            custom_item_id: t.custom_item_id,
            custom_id: t.custom_id,
            parent_id: t.parent,
            assignee_ids: t.assignees.into_iter().map(|a| a.id).collect(),
        }
    }
}
