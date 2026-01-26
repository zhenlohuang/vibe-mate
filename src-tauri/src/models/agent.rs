use serde::{Deserialize, Serialize};

use crate::agents::agent_metadata;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    ClaudeCode,
    Codex,
    GeminiCLI,
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
pub struct CodingAgent {
    pub agent_type: AgentType,
    pub name: String,
    pub version: Option<String>,
    pub status: AgentStatus,
    pub executable_path: Option<String>,
    pub config_path: Option<String>,
    pub auth_path: Option<String>,
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
        }
    }
}
