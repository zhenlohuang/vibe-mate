mod claude_code;
mod codex;
mod gemini_cli;
mod antigravity;
pub(crate) mod auth;

use std::process::Command;
use std::time::Duration;

use tokio::process::Command as TokioCommand;

use crate::models::{AgentProviderType, AgentQuota, AgentType};

pub use antigravity::AntigravityAgent;
pub use claude_code::ClaudeCodeAgent;
pub use codex::CodexAgent;
pub use gemini_cli::GeminiCliAgent;
pub use auth::{AgentAuthContext, AgentAuthError, AuthFlowStart};

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
    #[allow(dead_code)] // kept for sync trait impls; discover uses check_binary_installed_and_version
    fn is_installed(&self) -> bool;
}

#[allow(dead_code)] // kept for sync trait impls; discover uses check_binary_installed_and_version
pub(crate) fn binary_is_installed(binary: &str) -> bool {
    Command::new(binary)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Run `binary --version` once with a timeout; returns (is_installed, version_string).
/// Used by discover_agents to avoid running the same command twice per agent.
const BINARY_CHECK_TIMEOUT_SECS: u64 = 2;

pub async fn check_binary_installed_and_version(binary: &str) -> (bool, Option<String>) {
    let output = tokio::time::timeout(
        Duration::from_secs(BINARY_CHECK_TIMEOUT_SECS),
        TokioCommand::new(binary).arg("--version").output(),
    )
    .await;

    let output = match output {
        Ok(Ok(out)) => out,
        _ => return (false, None),
    };

    if !output.status.success() {
        return (false, None);
    }

    let version = String::from_utf8_lossy(&output.stdout)
        .trim()
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();

    if version.is_empty() {
        (true, None)
    } else {
        (true, Some(version))
    }
}

static ANTIGRAVITY_AGENT: AntigravityAgent = AntigravityAgent;
static CLAUDE_CODE_AGENT: ClaudeCodeAgent = ClaudeCodeAgent;
static CODEX_AGENT: CodexAgent = CodexAgent;
static GEMINI_CLI_AGENT: GeminiCliAgent = GeminiCliAgent;

pub fn all_agent_definitions() -> Vec<&'static dyn CodingAgentDefinition> {
    vec![
        &CLAUDE_CODE_AGENT,
        &CODEX_AGENT,
        &GEMINI_CLI_AGENT,
        &ANTIGRAVITY_AGENT,
    ]
}

pub fn agent_definition(agent_type: &AgentType) -> &'static dyn CodingAgentDefinition {
    match agent_type {
        AgentType::ClaudeCode => &CLAUDE_CODE_AGENT,
        AgentType::Codex => &CODEX_AGENT,
        AgentType::GeminiCLI => &GEMINI_CLI_AGENT,
        AgentType::Antigravity => &ANTIGRAVITY_AGENT,
    }
}

pub fn agent_metadata(agent_type: &AgentType) -> &'static AgentMetadata {
    agent_definition(agent_type).metadata()
}

pub fn start_agent_auth_flow(
    agent_type: &AgentProviderType,
    state: &str,
) -> Result<AuthFlowStart, AgentAuthError> {
    match agent_type {
        AgentProviderType::Codex => codex::start_auth_flow(state),
        AgentProviderType::ClaudeCode => claude_code::start_auth_flow(state),
        AgentProviderType::GeminiCli => gemini_cli::start_auth_flow(state),
        AgentProviderType::Antigravity => antigravity::start_auth_flow(state),
    }
}

pub async fn complete_agent_auth(
    ctx: &AgentAuthContext,
    agent_type: &AgentProviderType,
    state: &str,
    code: &str,
    code_verifier: &str,
) -> Result<(), AgentAuthError> {
    match agent_type {
        AgentProviderType::Codex => {
            codex::complete_auth(ctx, agent_type, state, code, code_verifier).await
        }
        AgentProviderType::ClaudeCode => {
            claude_code::complete_auth(ctx, agent_type, state, code, code_verifier).await
        }
        AgentProviderType::GeminiCli => {
            gemini_cli::complete_auth(ctx, agent_type, state, code, code_verifier).await
        }
        AgentProviderType::Antigravity => {
            antigravity::complete_auth(ctx, agent_type, state, code, code_verifier).await
        }
    }
}

pub async fn get_agent_quota(
    ctx: &AgentAuthContext,
    agent_type: &AgentProviderType,
) -> Result<AgentQuota, AgentAuthError> {
    match agent_type {
        AgentProviderType::Codex => codex::get_quota(ctx, agent_type).await,
        AgentProviderType::ClaudeCode => claude_code::get_quota(ctx, agent_type).await,
        AgentProviderType::GeminiCli => gemini_cli::get_quota(ctx, agent_type).await,
        AgentProviderType::Antigravity => antigravity::get_quota(ctx, agent_type).await,
    }
}
