use crate::agents::{binary_is_installed, resolve_binary_version, AgentMetadata, CodingAgentDefinition};
use crate::models::AgentType;

pub struct GeminiCliAgent;

impl GeminiCliAgent {
    pub const METADATA: AgentMetadata = AgentMetadata {
        agent_type: AgentType::GeminiCLI,
        name: "Gemini CLI",
        binary: "gemini",
        default_config_file: "~/.gemini/settings.json",
        default_auth_file: "~/.gemini/credentials.json",
    };
}

impl CodingAgentDefinition for GeminiCliAgent {
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
