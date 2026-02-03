mod claude_code;
mod codex;
mod gemini_cli;
mod antigravity;
pub(crate) mod auth;

use std::process::Command;

use crate::models::{AgentProviderType, AgentQuota, AgentType, Provider};

pub use claude_code::ClaudeCodeAgent;
pub use codex::CodexAgent;
pub use gemini_cli::GeminiCliAgent;
pub use auth::{AgentAuth, AgentAuthContext, AgentAuthError, AuthFlowStart};

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
    provider_id: &str,
    agent_type: &AgentProviderType,
    state: &str,
    code: &str,
    code_verifier: &str,
) -> Result<(), AgentAuthError> {
    match agent_type {
        AgentProviderType::Codex => {
            codex::complete_auth(ctx, provider_id, state, code, code_verifier).await
        }
        AgentProviderType::ClaudeCode => {
            claude_code::complete_auth(ctx, provider_id, state, code, code_verifier).await
        }
        AgentProviderType::GeminiCli => {
            gemini_cli::complete_auth(ctx, provider_id, state, code, code_verifier).await
        }
        AgentProviderType::Antigravity => {
            antigravity::complete_auth(ctx, provider_id, state, code, code_verifier).await
        }
    }
}

pub async fn get_agent_quota(
    ctx: &AgentAuthContext,
    provider: &Provider,
    agent_type: &AgentProviderType,
) -> Result<AgentQuota, AgentAuthError> {
    match agent_type {
        AgentProviderType::Codex => codex::get_quota(ctx, provider).await,
        AgentProviderType::ClaudeCode => claude_code::get_quota(ctx, provider).await,
        AgentProviderType::GeminiCli => gemini_cli::get_quota(ctx, provider).await,
        AgentProviderType::Antigravity => antigravity::get_quota(ctx, provider).await,
    }
}

/// Get valid authentication for proxy requests to an Agent provider
pub async fn get_agent_auth(
    ctx: &AgentAuthContext,
    provider: &Provider,
    agent_type: &AgentProviderType,
) -> Result<AgentAuth, AgentAuthError> {
    match agent_type {
        AgentProviderType::Codex => codex::get_valid_auth(ctx, provider).await,
        AgentProviderType::ClaudeCode => claude_code::get_valid_auth(ctx, provider).await,
        AgentProviderType::GeminiCli => gemini_cli::get_valid_auth(ctx, provider).await,
        AgentProviderType::Antigravity => antigravity::get_valid_auth(ctx, provider).await,
    }
}
