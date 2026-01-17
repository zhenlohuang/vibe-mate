use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    ClaudeCode,
    Codex,
    GeminiCLI,
}

impl AgentType {
    pub fn all() -> Vec<AgentType> {
        vec![
            AgentType::ClaudeCode,
            AgentType::Codex,
            AgentType::GeminiCLI,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "Claude Code",
            AgentType::Codex => "Codex",
            AgentType::GeminiCLI => "Gemini CLI",
        }
    }

    pub fn detection_command(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "claude",
            AgentType::Codex => "codex",
            AgentType::GeminiCLI => "gemini",
        }
    }
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
        Self {
            name: agent_type.display_name().to_string(),
            agent_type,
            version: None,
            status: AgentStatus::NotInstalled,
            executable_path: None,
            config_path: None,
            auth_path: None,
        }
    }
}
