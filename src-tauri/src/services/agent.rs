use std::process::Command;
use std::fs;
use std::path::PathBuf;

use crate::models::{AgentType, CodingAgent};

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Command execution error: {0}")]
    CommandError(String),
}

pub struct AgentService;

impl AgentService {
    pub fn new() -> Self {
        Self
    }

    /// Discover all supported coding agents in the system
    pub async fn discover_agents(&self) -> Result<Vec<CodingAgent>, AgentError> {
        let mut agents = Vec::new();

        for agent_type in AgentType::all() {
            let agent = self.check_agent(&agent_type).await;
            agents.push(agent);
        }

        Ok(agents)
    }

    /// Check a specific agent's status
    async fn check_agent(&self, agent_type: &AgentType) -> CodingAgent {
        CodingAgent::new(agent_type.clone())
    }

    /// Get version information for an agent (synchronous)
    fn get_version_sync(&self, agent_type: &AgentType) -> Option<String> {
        let command = agent_type.detection_command();

        let output = Command::new(command).arg("--version").output().ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            // Extract version number from output
            let version = version
                .lines()
                .next()
                .unwrap_or(&version)
                .trim()
                .to_string();
            if !version.is_empty() {
                return Some(version);
            }
        }

        None
    }

    /// Get version information for an agent
    pub async fn get_version(&self, agent_type: &AgentType) -> Option<String> {
        self.get_version_sync(agent_type)
    }


    /// Check status of a specific agent
    pub async fn check_status(&self, agent_type: &AgentType) -> Result<CodingAgent, AgentError> {
        Ok(self.check_agent(agent_type).await)
    }

    /// Open the login flow for an agent
    pub async fn open_login(&self, agent_type: &AgentType) -> Result<(), AgentError> {
        let command = agent_type.detection_command();

        // Try to open the login command in a new terminal
        // This is platform-specific
        #[cfg(target_os = "macos")]
        {
            let script = format!(
                r#"tell application "Terminal"
                    do script "{} auth login"
                    activate
                end tell"#,
                command
            );
            Command::new("osascript")
                .args(["-e", &script])
                .spawn()
                .map_err(|e| AgentError::CommandError(e.to_string()))?;
        }

        #[cfg(target_os = "linux")]
        {
            // Try common terminal emulators
            let terminals = ["gnome-terminal", "konsole", "xterm"];
            for term in &terminals {
                if Command::new("which")
                    .arg(term)
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
                {
                    Command::new(term)
                        .args(["--", command, "auth", "login"])
                        .spawn()
                        .map_err(|e| AgentError::CommandError(e.to_string()))?;
                    break;
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(["/c", "start", "cmd", "/k", command, "auth", "login"])
                .spawn()
                .map_err(|e| AgentError::CommandError(e.to_string()))?;
        }

        Ok(())
    }

    /// Get the config file path for an agent
    fn get_config_path(&self, agent_type: &AgentType) -> Option<PathBuf> {
        let home_dir = dirs::home_dir()?;
        
        let config_path = match agent_type {
            AgentType::ClaudeCode => home_dir.join(".claude").join("settings.json"),
            AgentType::Codex => home_dir.join(".codex").join("config.toml"),
            AgentType::GeminiCLI => home_dir.join(".gemini").join("settings.json"),
        };
        
        Some(config_path)
    }

    fn resolve_config_path(
        &self,
        agent_type: &AgentType,
        config_path: Option<String>,
    ) -> Option<PathBuf> {
        if let Some(path) = config_path {
            return Some(self.expand_tilde_path(path));
        }
        self.get_config_path(agent_type)
    }

    fn expand_tilde_path(&self, path: String) -> PathBuf {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(home_dir) = dirs::home_dir() {
                return home_dir.join(stripped);
            }
        }
        if path == "~" {
            if let Some(home_dir) = dirs::home_dir() {
                return home_dir;
            }
        }
        PathBuf::from(path)
    }

    /// Read configuration file for an agent
    pub async fn read_config(
        &self,
        agent_type: &AgentType,
        config_path: Option<String>,
    ) -> Result<String, AgentError> {
        let config_path = self
            .resolve_config_path(agent_type, config_path)
            .ok_or_else(|| AgentError::CommandError("Could not determine home directory".to_string()))?;
        
        if !config_path.exists() {
            return Err(AgentError::CommandError(format!(
                "Config file not found: {}",
                config_path.display()
            )));
        }
        
        fs::read_to_string(&config_path)
            .map_err(|e| AgentError::CommandError(format!("Failed to read config file: {}", e)))
    }

    /// Save configuration file for an agent
    pub async fn save_config(
        &self,
        agent_type: &AgentType,
        content: String,
        config_path: Option<String>,
    ) -> Result<(), AgentError> {
        let config_path = self
            .resolve_config_path(agent_type, config_path)
            .ok_or_else(|| AgentError::CommandError("Could not determine home directory".to_string()))?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AgentError::CommandError(format!("Failed to create config directory: {}", e)))?;
        }
        
        fs::write(&config_path, content)
            .map_err(|e| AgentError::CommandError(format!("Failed to save config file: {}", e)))
    }
}

impl Default for AgentService {
    fn default() -> Self {
        Self::new()
    }
}
