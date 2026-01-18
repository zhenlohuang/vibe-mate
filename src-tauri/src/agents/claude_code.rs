use crate::agents::{binary_is_installed, resolve_binary_version, AgentMetadata, CodingAgentDefinition};
use crate::models::AgentType;

pub struct ClaudeCodeAgent;

impl ClaudeCodeAgent {
    pub const METADATA: AgentMetadata = AgentMetadata {
        agent_type: AgentType::ClaudeCode,
        name: "Claude Code",
        binary: "claude",
        default_config_file: "~/.claude/settings.json",
        default_auth_file: "~/.claude/credentials.json",
    };
}

impl CodingAgentDefinition for ClaudeCodeAgent {
    fn metadata(&self) -> &'static AgentMetadata {
        &Self::METADATA
    }

    fn is_installed(&self) -> bool {
        binary_is_installed(Self::METADATA.binary)
    }

    fn get_version(&self) -> Option<String> {
        resolve_binary_version(Self::METADATA.binary)
    }
}
