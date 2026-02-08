use std::fs;
use std::path::PathBuf;

use futures_util::future::join_all;
use crate::agents::{agent_metadata, all_agent_definitions, check_binary_installed_and_version};
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

    /// Discover all supported coding agents in the system (parallel detection).
    pub async fn discover_agents(&self) -> Result<Vec<CodingAgent>, AgentError> {
        let agent_types: Vec<AgentType> = all_agent_definitions()
            .into_iter()
            .map(|definition| definition.metadata().agent_type.clone())
            .collect();

        let agents = join_all(
            agent_types
                .iter()
                .map(|agent_type| self.check_agent(agent_type)),
        )
        .await;

        Ok(agents)
    }

    /// Check a specific agent's status (single binary --version call with timeout).
    async fn check_agent(&self, agent_type: &AgentType) -> CodingAgent {
        let metadata = agent_metadata(agent_type);
        let (is_installed, version) = check_binary_installed_and_version(metadata.binary).await;

        let mut agent = CodingAgent::new(agent_type.clone());
        agent.status = if is_installed {
            AgentStatus::Installed
        } else {
            AgentStatus::NotInstalled
        };
        agent.version = version;
        agent
    }

    /// Check status of a specific agent
    pub async fn check_status(&self, agent_type: &AgentType) -> Result<CodingAgent, AgentError> {
        Ok(self.check_agent(agent_type).await)
    }

    /// Get the config file path for an agent
    fn get_config_path(&self, agent_type: &AgentType) -> Option<PathBuf> {
        let metadata = agent_metadata(agent_type);
        Some(self.expand_tilde_path(metadata.default_config_file.to_string()))
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
