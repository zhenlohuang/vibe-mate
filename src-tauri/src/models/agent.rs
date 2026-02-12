use serde::{Deserialize, Serialize};

use crate::agents::agent_metadata;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    ClaudeCode,
    Codex,
    GeminiCLI,
    Antigravity,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Installed,
    NotInstalled,
    Authenticated,
    NotAuthenticated,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::NotInstalled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct CodingAgent {
    pub agent_type: AgentType,
    pub name: String,
    pub version: Option<String>,
    pub status: AgentStatus,
    pub executable_path: Option<String>,
    pub config_path: Option<String>,
    pub auth_path: Option<String>,
    /// Whether to show this agent on the Dashboard. Default true when new.
    pub featured: bool,
    /// Whether VibeMate proxy auto-config is enabled for this agent.
    pub proxy_enabled: bool,
}

impl Default for CodingAgent {
    fn default() -> Self {
        Self {
            agent_type: AgentType::ClaudeCode,
            name: String::new(),
            version: None,
            status: AgentStatus::default(),
            executable_path: None,
            config_path: None,
            auth_path: None,
            featured: true,
            proxy_enabled: false,
        }
    }
}

impl CodingAgent {
    pub fn new(agent_type: AgentType) -> Self {
        let metadata = agent_metadata(&agent_type);
        Self {
            name: metadata.name.to_string(),
            agent_type,
            version: None,
            status: AgentStatus::NotInstalled,
            executable_path: Some(metadata.binary.to_string()),
            config_path: Some(metadata.default_config_file.to_string()),
            auth_path: Some(metadata.default_auth_file.to_string()),
            featured: true,
            proxy_enabled: false,
        }
    }
}
