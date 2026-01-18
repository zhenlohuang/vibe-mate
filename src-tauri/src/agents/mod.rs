mod claude_code;
mod codex;
mod gemini_cli;

use std::process::Command;

use crate::models::AgentType;

pub use claude_code::ClaudeCodeAgent;
pub use codex::CodexAgent;
pub use gemini_cli::GeminiCliAgent;

#[derive(Debug, Clone)]
pub struct AgentMetadata {
    pub agent_type: AgentType,
    pub name: &'static str,
    pub binary: &'static str,
    pub default_config_file: &'static str,
    pub default_auth_file: &'static str,
}

pub trait CodingAgentDefinition {
    fn metadata(&self) -> &'static AgentMetadata;
    fn is_installed(&self) -> bool;
    fn get_version(&self) -> Option<String>;
}

pub(crate) fn resolve_binary_version(binary: &str) -> Option<String> {
    let output = Command::new(binary).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();

    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

pub(crate) fn binary_is_installed(binary: &str) -> bool {
    Command::new(binary)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

static CLAUDE_CODE_AGENT: ClaudeCodeAgent = ClaudeCodeAgent;
static CODEX_AGENT: CodexAgent = CodexAgent;
static GEMINI_CLI_AGENT: GeminiCliAgent = GeminiCliAgent;

pub fn all_agent_definitions() -> Vec<&'static dyn CodingAgentDefinition> {
    vec![&CLAUDE_CODE_AGENT, &CODEX_AGENT, &GEMINI_CLI_AGENT]
}

pub fn agent_definition(agent_type: &AgentType) -> &'static dyn CodingAgentDefinition {
    match agent_type {
        AgentType::ClaudeCode => &CLAUDE_CODE_AGENT,
        AgentType::Codex => &CODEX_AGENT,
        AgentType::GeminiCLI => &GEMINI_CLI_AGENT,
    }
}

pub fn agent_metadata(agent_type: &AgentType) -> &'static AgentMetadata {
    agent_definition(agent_type).metadata()
}
