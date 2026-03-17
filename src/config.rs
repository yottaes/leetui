use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub workspace_dir: String,
    pub language: String,
    pub editor: String,
    #[serde(default)]
    pub leetcode_session: Option<String>,
    #[serde(default)]
    pub csrf_token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workspace_dir: "~/leetcode".to_string(),
            language: "rust".to_string(),
            editor: "vim".to_string(),
            leetcode_session: None,
            csrf_token: None,
        }
    }
}

impl Config {
    /// Creates a default config, saves it to disk, and ensures the workspace directory exists.
    pub fn create_default() -> Result<Config> {
        let config = Config::default();
        config.save()?;
        let workspace = config.expanded_workspace();
        std::fs::create_dir_all(&workspace)
            .with_context(|| format!("Failed to create workspace dir {}", workspace.display()))?;
        Ok(config)
    }

    pub fn is_authenticated(&self) -> bool {
        self.leetcode_session.as_ref().is_some_and(|s| !s.is_empty())
            && self.csrf_token.as_ref().is_some_and(|s| !s.is_empty())
    }

    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".leetcode-cli")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Result<Option<Config>> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config.toml")?;
        Ok(Some(config))
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create config dir {}", dir.display()))?;
        let path = Self::config_path();
        let contents =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;
        std::fs::write(&path, contents)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;
        Ok(())
    }

    pub fn expanded_workspace(&self) -> PathBuf {
        let expanded = if self.workspace_dir.starts_with('~') {
            let home = dirs::home_dir().expect("Could not find home directory");
            home.join(self.workspace_dir.strip_prefix("~/").unwrap_or(""))
        } else {
            PathBuf::from(&self.workspace_dir)
        };
        expanded
    }
}
