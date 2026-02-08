mod claude_code;
mod codex;
mod gemini_cli;
mod antigravity;
pub(crate) mod auth;

use std::path::PathBuf;
use std::process::Command;

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
}

/// Build a list of candidate directories where CLI tools are commonly installed.
/// When the app is packaged (e.g. macOS .app bundle), the inherited PATH is
/// minimal (`/usr/bin:/bin:/usr/sbin:/sbin`), so we must also look in well-known
/// locations to find binaries like `claude`, `codex`, `gemini`, etc.
fn common_binary_search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    #[cfg(unix)]
    {
        // System-wide locations
        dirs.push(PathBuf::from("/usr/local/bin"));
        dirs.push(PathBuf::from("/usr/bin"));

        // macOS Homebrew
        #[cfg(target_os = "macos")]
        {
            dirs.push(PathBuf::from("/opt/homebrew/bin")); // Apple Silicon
            dirs.push(PathBuf::from("/usr/local/Homebrew/bin")); // Intel
        }

        // Linux snap / flatpak
        #[cfg(target_os = "linux")]
        {
            dirs.push(PathBuf::from("/snap/bin"));
        }

        if let Some(home) = dirs::home_dir() {
            // npm global installs
            dirs.push(home.join(".npm/bin"));
            dirs.push(home.join(".npm-global/bin"));
            // pnpm global
            dirs.push(home.join(".local/share/pnpm"));
            // yarn global
            dirs.push(home.join(".yarn/bin"));
            // bun global
            dirs.push(home.join(".bun/bin"));
            // cargo installs
            dirs.push(home.join(".cargo/bin"));
            // pipx / user local
            dirs.push(home.join(".local/bin"));
            // Antigravity CLI
            dirs.push(home.join(".antigravity/antigravity/bin"));
            // nvm-managed Node.js — scan all installed versions
            let nvm_versions = home.join(".nvm/versions/node");
            if let Ok(entries) = std::fs::read_dir(&nvm_versions) {
                for entry in entries.flatten() {
                    let bin = entry.path().join("bin");
                    if bin.is_dir() {
                        dirs.push(bin);
                    }
                }
            }
            // fnm-managed Node.js
            let fnm_versions = home.join(".local/share/fnm/node-versions");
            if let Ok(entries) = std::fs::read_dir(&fnm_versions) {
                for entry in entries.flatten() {
                    let bin = entry.path().join("installation/bin");
                    if bin.is_dir() {
                        dirs.push(bin);
                    }
                }
            }
            // Volta-managed Node.js
            dirs.push(home.join(".volta/bin"));
        }

        // macOS .app bundles may ship their own CLI binaries
        #[cfg(target_os = "macos")]
        {
            // Antigravity.app ships a CLI binary inside the app bundle
            dirs.push(PathBuf::from(
                "/Applications/Antigravity.app/Contents/Resources/app/bin",
            ));
        }
    }

    #[cfg(windows)]
    {
        if let Some(home) = dirs::home_dir() {
            // npm global installs (default on Windows)
            dirs.push(home.join("AppData\\Roaming\\npm"));
            // pnpm global
            dirs.push(home.join("AppData\\Local\\pnpm"));
            // yarn global
            dirs.push(home.join("AppData\\Local\\Yarn\\bin"));
            // bun global
            dirs.push(home.join(".bun\\bin"));
            // cargo installs
            dirs.push(home.join(".cargo\\bin"));
            // Scoop
            dirs.push(home.join("scoop\\shims"));
            // Volta
            dirs.push(home.join(".volta\\bin"));
            // Antigravity CLI
            dirs.push(home.join(".antigravity\\antigravity\\bin"));
            // nvm-windows
            let nvm_dir = home.join("AppData\\Roaming\\nvm");
            if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs.push(path);
                    }
                }
            }
            // fnm on Windows
            let fnm_versions = home.join("AppData\\Roaming\\fnm\\node-versions");
            if let Ok(entries) = std::fs::read_dir(&fnm_versions) {
                for entry in entries.flatten() {
                    let bin = entry.path().join("installation");
                    if bin.is_dir() {
                        dirs.push(bin);
                    }
                }
            }
        }
        // Common system paths
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            dirs.push(PathBuf::from(&program_files).join("nodejs"));
            // Antigravity on Windows
            dirs.push(PathBuf::from(&program_files).join("Antigravity\\resources\\app\\bin"));
        }
        if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
            dirs.push(PathBuf::from(&program_files_x86).join("nodejs"));
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            // Antigravity per-user install on Windows
            dirs.push(PathBuf::from(&local_app_data).join("Programs\\Antigravity\\resources\\app\\bin"));
        }
    }

    dirs
}

/// Resolve the full path of a binary by first checking PATH, then searching
/// common installation directories. Returns `None` if not found anywhere.
fn resolve_binary_path(binary: &str) -> Option<PathBuf> {
    // Check if binary already contains a path separator — treat as absolute/relative
    let binary_path = PathBuf::from(binary);
    if binary_path.components().count() > 1 && binary_path.exists() {
        return Some(binary_path);
    }

    // Try PATH first via `which` (Unix) / `where` (Windows)
    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("which").arg(binary).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }
    }
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("where").arg(binary).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }
    }

    // Fallback: search common installation directories
    let search_dirs = common_binary_search_dirs();

    #[cfg(windows)]
    let binary_names = [format!("{}.cmd", binary), format!("{}.exe", binary), binary.to_string()];
    #[cfg(not(windows))]
    let binary_names = [binary.to_string()];

    for dir in &search_dirs {
        for name in &binary_names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Check whether a binary is installed by resolving its path.
///
/// When the app runs as a packaged bundle (e.g. macOS .app), the process PATH is
/// minimal. We therefore resolve the binary via [`resolve_binary_path`] which also
/// searches well-known installation directories.
pub(crate) fn is_binary_installed(binary: &str) -> bool {
    resolve_binary_path(binary).is_some()
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
