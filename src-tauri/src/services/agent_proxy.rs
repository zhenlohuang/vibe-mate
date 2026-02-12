use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Map as JsonMap, Value as JsonValue};
use tokio::fs;
use toml::Value as TomlValue;

use crate::agents::agent_metadata;
use crate::models::{AgentType, CodingAgent};
use crate::storage::ConfigStore;

const LEGACY_CLAUDE_PROXY_MARKER_KEY: &str = "proxyEnabled";
const CLAUDE_ENV_KEY: &str = "env";
const CLAUDE_BASE_URL_KEY: &str = "ANTHROPIC_BASE_URL";
const LEGACY_CODEX_PROXY_MARKER_KEY: &str = "proxy_enabled";
const CODEX_ENV_KEY: &str = "env";
const CODEX_BASE_URL_KEY: &str = "OPENAI_BASE_URL";

#[derive(Debug, thiserror::Error)]
pub enum AgentProxyError {
    #[error("Proxy auto-config is not supported for agent type: {0:?}")]
    UnsupportedAgent(AgentType),
    #[error("Could not determine home directory")]
    HomeDirectoryUnavailable,
    #[error("Invalid config format: {0}")]
    InvalidConfigFormat(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML parse error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::StorageError),
}

pub struct AgentProxyService {
    store: Arc<ConfigStore>,
}

impl AgentProxyService {
    pub fn new(store: Arc<ConfigStore>) -> Self {
        Self { store }
    }

    pub async fn is_proxy_enabled(&self, agent_type: &AgentType) -> Result<bool, AgentProxyError> {
        if !is_proxy_supported_agent(agent_type) {
            return Err(AgentProxyError::UnsupportedAgent(agent_type.clone()));
        }

        let config = self.store.get_config().await;
        Ok(config
            .coding_agents
            .iter()
            .find(|agent| agent.agent_type == *agent_type)
            .map(|agent| agent.proxy_enabled)
            .unwrap_or(false))
    }

    pub async fn set_proxy_enabled(
        &self,
        agent_type: &AgentType,
        enabled: bool,
    ) -> Result<(), AgentProxyError> {
        if !is_proxy_supported_agent(agent_type) {
            return Err(AgentProxyError::UnsupportedAgent(agent_type.clone()));
        }

        let config = self.store.get_config().await;
        let port = config.app.port;
        let config_path = resolve_agent_config_path(agent_type)?;

        match agent_type {
            AgentType::ClaudeCode => {
                self.write_claude_proxy_enabled(&config_path, enabled, port)
                    .await?
            }
            AgentType::Codex => {
                self.write_codex_proxy_enabled(&config_path, enabled, port)
                    .await?
            }
            _ => return Err(AgentProxyError::UnsupportedAgent(agent_type.clone())),
        }

        self.persist_proxy_enabled(agent_type, enabled).await?;
        Ok(())
    }

    async fn write_claude_proxy_enabled(
        &self,
        path: &Path,
        enabled: bool,
        port: u16,
    ) -> Result<(), AgentProxyError> {
        let mut root = read_json_or_default(path).await?;
        let root_obj = root.as_object_mut().ok_or_else(|| {
            AgentProxyError::InvalidConfigFormat(
                "Claude config root must be a JSON object".to_string(),
            )
        })?;
        // Legacy cleanup: status is persisted in ~/.vibemate/settings.json now.
        root_obj.remove(LEGACY_CLAUDE_PROXY_MARKER_KEY);

        if enabled {
            let env_value = root_obj
                .entry(CLAUDE_ENV_KEY.to_string())
                .or_insert_with(|| JsonValue::Object(JsonMap::new()));
            if !env_value.is_object() {
                *env_value = JsonValue::Object(JsonMap::new());
            }
            if let Some(env_obj) = env_value.as_object_mut() {
                env_obj.insert(
                    CLAUDE_BASE_URL_KEY.to_string(),
                    JsonValue::String(format!("http://localhost:{port}/api/anthropic")),
                );
            }
        } else {
            let mut remove_env = false;
            if let Some(env_value) = root_obj.get_mut(CLAUDE_ENV_KEY) {
                if let Some(env_obj) = env_value.as_object_mut() {
                    env_obj.remove(CLAUDE_BASE_URL_KEY);
                    remove_env = env_obj.is_empty();
                } else {
                    remove_env = true;
                }
            }
            if remove_env {
                root_obj.remove(CLAUDE_ENV_KEY);
            }
        }

        write_json(path, &root).await
    }

    async fn write_codex_proxy_enabled(
        &self,
        path: &Path,
        enabled: bool,
        port: u16,
    ) -> Result<(), AgentProxyError> {
        let mut root = read_toml_or_default(path).await?;
        let root_table = root.as_table_mut().ok_or_else(|| {
            AgentProxyError::InvalidConfigFormat("Codex config root must be a TOML table".to_string())
        })?;
        // Legacy cleanup: status is persisted in ~/.vibemate/settings.json now.
        root_table.remove(LEGACY_CODEX_PROXY_MARKER_KEY);

        if enabled {
            let env_value = root_table
                .entry(CODEX_ENV_KEY.to_string())
                .or_insert_with(|| TomlValue::Table(toml::map::Map::new()));
            if !env_value.is_table() {
                *env_value = TomlValue::Table(toml::map::Map::new());
            }
            if let Some(env_table) = env_value.as_table_mut() {
                env_table.insert(
                    CODEX_BASE_URL_KEY.to_string(),
                    TomlValue::String(format!("http://localhost:{port}/api/openai/v1")),
                );
            }
        } else {
            let mut remove_env = false;
            if let Some(env_value) = root_table.get_mut(CODEX_ENV_KEY) {
                if let Some(env_table) = env_value.as_table_mut() {
                    env_table.remove(CODEX_BASE_URL_KEY);
                    remove_env = env_table.is_empty();
                } else {
                    remove_env = true;
                }
            }
            if remove_env {
                root_table.remove(CODEX_ENV_KEY);
            }
        }

        write_toml(path, &root).await
    }

    async fn persist_proxy_enabled(
        &self,
        agent_type: &AgentType,
        enabled: bool,
    ) -> Result<(), AgentProxyError> {
        let target_type = agent_type.clone();
        self.store
            .update(move |config| {
                if let Some(entry) = config
                    .coding_agents
                    .iter_mut()
                    .find(|agent| agent.agent_type == target_type)
                {
                    entry.proxy_enabled = enabled;
                    return;
                }

                let mut new_entry = CodingAgent::new(target_type.clone());
                new_entry.proxy_enabled = enabled;
                config.coding_agents.push(new_entry);
            })
            .await?;
        Ok(())
    }
}

fn resolve_agent_config_path(agent_type: &AgentType) -> Result<PathBuf, AgentProxyError> {
    if !matches!(agent_type, AgentType::ClaudeCode | AgentType::Codex) {
        return Err(AgentProxyError::UnsupportedAgent(agent_type.clone()));
    }

    let metadata = agent_metadata(agent_type);
    expand_tilde_path(metadata.default_config_file)
}

fn is_proxy_supported_agent(agent_type: &AgentType) -> bool {
    matches!(agent_type, AgentType::ClaudeCode | AgentType::Codex)
}

fn expand_tilde_path(path: &str) -> Result<PathBuf, AgentProxyError> {
    if path == "~" {
        return dirs::home_dir().ok_or(AgentProxyError::HomeDirectoryUnavailable);
    }

    if let Some(stripped) = path.strip_prefix("~/") {
        let home_dir = dirs::home_dir().ok_or(AgentProxyError::HomeDirectoryUnavailable)?;
        return Ok(home_dir.join(stripped));
    }

    Ok(PathBuf::from(path))
}

async fn ensure_parent_dir(path: &Path) -> Result<(), AgentProxyError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

async fn read_json_or_default(path: &Path) -> Result<JsonValue, AgentProxyError> {
    if !fs::try_exists(path).await? {
        return Ok(JsonValue::Object(JsonMap::new()));
    }
    let content = fs::read_to_string(path).await?;
    Ok(serde_json::from_str(&content)?)
}

async fn write_json(path: &Path, value: &JsonValue) -> Result<(), AgentProxyError> {
    ensure_parent_dir(path).await?;
    let content = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{content}\n")).await?;
    Ok(())
}

async fn read_toml_or_default(path: &Path) -> Result<TomlValue, AgentProxyError> {
    if !fs::try_exists(path).await? {
        return Ok(TomlValue::Table(toml::map::Map::new()));
    }
    let content = fs::read_to_string(path).await?;
    Ok(toml::from_str(&content)?)
}

async fn write_toml(path: &Path, value: &TomlValue) -> Result<(), AgentProxyError> {
    ensure_parent_dir(path).await?;
    let content = toml::to_string_pretty(value)?;
    fs::write(path, format!("{content}\n")).await?;
    Ok(())
}
