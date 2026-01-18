use crate::agents::{binary_is_installed, resolve_binary_version, AgentMetadata, CodingAgentDefinition};
use crate::models::AgentType;

pub struct CodexAgent;

impl CodexAgent {
    pub const METADATA: AgentMetadata = AgentMetadata {
        agent_type: AgentType::Codex,
        name: "Codex",
        binary: "codex",
        default_config_file: "~/.codex/config.toml",
        default_auth_file: "~/.codex/auth.json",
    };
}

impl CodingAgentDefinition for CodexAgent {
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
