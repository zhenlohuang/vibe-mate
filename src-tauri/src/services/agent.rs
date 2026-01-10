use std::process::Command;

use crate::models::{AgentStatus, AgentType, CodingAgent};

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
        let mut agent = CodingAgent::new(agent_type.clone());

        // Check if the agent is installed
        if let Some(path) = self.find_executable(agent_type) {
            agent.executable_path = Some(path);
            agent.status = AgentStatus::Installed;

            // Try to get version
            if let Some(version) = self.get_version_sync(agent_type) {
                agent.version = Some(version);
            }

            // Check authentication status
            if self.check_auth_status_sync(agent_type) {
                agent.status = AgentStatus::Authenticated;
            } else {
                agent.status = AgentStatus::NotAuthenticated;
            }
        }

        agent
    }

    /// Find the executable path for an agent
    fn find_executable(&self, agent_type: &AgentType) -> Option<String> {
        let command = agent_type.detection_command();

        #[cfg(unix)]
        let which_cmd = "which";
        #[cfg(windows)]
        let which_cmd = "where";

        let output = Command::new(which_cmd).arg(command).output().ok()?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }

        None
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

    /// Check authentication status (synchronous)
    fn check_auth_status_sync(&self, agent_type: &AgentType) -> bool {
        // For now, we'll check for config files as a proxy for authentication
        // In a real implementation, you'd check for actual auth tokens

        let home_dir = dirs::home_dir();
        if home_dir.is_none() {
            return false;
        }
        let home = home_dir.unwrap();

        match agent_type {
            AgentType::ClaudeCode => {
                // Check for Claude Code config
                let config_path = home.join(".claude");
                config_path.exists()
            }
            AgentType::GeminiCLI => {
                // Check for Gemini CLI config
                let config_path = home.join(".gemini");
                config_path.exists()
            }
        }
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
}

impl Default for AgentService {
    fn default() -> Self {
        Self::new()
    }
}

