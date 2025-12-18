//! Configuration handling for clickup-tui
//!
//! Stores API token and user settings in XDG-compliant locations.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// ClickUp API token
    pub api_token: String,
    /// ClickUp user ID (numeric)
    pub user_id: String,
    /// Auto-refresh on startup
    #[serde(default = "default_auto_refresh")]
    pub auto_refresh: bool,
}

fn default_auto_refresh() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_token: String::new(),
            user_id: String::new(),
            auto_refresh: true,
        }
    }
}

impl Config {
    /// Get the config directory path (~/.config/clickup-tui on all platforms)
    pub fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory")?;
        Ok(PathBuf::from(home).join(".config").join("clickup-tui"))
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Get the local state file path (for pins, snoozes, etc.)
    pub fn state_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("local_state.json"))
    }

    /// Get the cache file path (for cached tasks)
    pub fn cache_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("tasks_cache.json"))
    }

    /// Load config from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            // Create default config
            let config = Self::default();
            config.save()?;

            anyhow::bail!(
                "Config file created at {}. Please edit it to add your ClickUp API token and user ID.",
                path.display()
            );
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;

        // Validate required fields
        if config.api_token.is_empty() {
            anyhow::bail!("api_token is required in config file: {}", path.display());
        }
        if config.user_id.is_empty() {
            anyhow::bail!("user_id is required in config file: {}", path.display());
        }

        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let dir = path.parent().unwrap();

        // Create directory if needed
        fs::create_dir_all(dir)
            .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;

        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        // Add helpful comments
        let content_with_comments = format!(
            "# ClickUp TUI Configuration\n\
             # \n\
             # Get your API token from: ClickUp Settings > Apps > API Token\n\
             # Find your user ID by running: clickup-tui --show-user-id\n\
             # Or check the ClickUp MCP server output\n\
             \n\
             {content}"
        );

        fs::write(&path, content_with_comments)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.api_token.is_empty());
        assert!(config.user_id.is_empty());
        assert!(config.auto_refresh);
    }
}
