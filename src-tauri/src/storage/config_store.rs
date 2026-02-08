use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

use crate::models::{AgentType, CodingAgent, VibeMateConfig};

const CONFIG_FILE: &str = "settings.json";

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct ConfigStore {
    config_dir: PathBuf,
    config: Arc<RwLock<VibeMateConfig>>,
}

impl ConfigStore {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            config: Arc::new(RwLock::new(VibeMateConfig::default())),
        }
    }

    /// Get configuration file path
    fn config_path(&self) -> PathBuf {
        self.config_dir.join(CONFIG_FILE)
    }

    /// Initialize storage (create directory and load configuration)
    pub async fn init(&self) -> Result<(), StorageError> {
        fs::create_dir_all(&self.config_dir).await?;
        self.load().await?;
        Ok(())
    }

    /// Load configuration from file. Migrates legacy config (e.g. drops Agent providers).
    pub async fn load(&self) -> Result<(), StorageError> {
        let path = self.config_path();
        let config = if path.exists() {
            let content = fs::read_to_string(&path).await?;
            let raw: serde_json::Value = serde_json::from_str(&content)?;

            match serde_json::from_value::<VibeMateConfig>(raw.clone()) {
                Ok(c) => c,
                Err(_) => {
                    // Legacy config may contain Agent providers with old type enum; keep only model providers
                    let app = raw
                        .get("app")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    let routing_rules = raw
                        .get("routingRules")
                        .or_else(|| raw.get("routing_rules"))
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    let providers: Vec<crate::models::Provider> = raw
                        .get("providers")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                                .collect()
                        })
                        .unwrap_or_default();
                    let coding_agents = raw
                        .get("codingAgents")
                        .or_else(|| raw.get("coding_agents"))
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    VibeMateConfig {
                        app,
                        providers,
                        routing_rules,
                        coding_agents,
                    }
                }
            }
        } else {
            VibeMateConfig::default()
        };
        *self.config.write().await = config;
        Ok(())
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<(), StorageError> {
        let path = self.config_path();
        let config = self.config.read().await;
        let content = serde_json::to_string_pretty(&*config)?;
        fs::write(&path, content).await?;
        Ok(())
    }

    /// Get complete configuration (read-only)
    pub async fn get_config(&self) -> VibeMateConfig {
        self.config.read().await.clone()
    }

    /// Update configuration and save
    pub async fn update<F>(&self, f: F) -> Result<(), StorageError>
    where
        F: FnOnce(&mut VibeMateConfig),
    {
        {
            let mut config = self.config.write().await;
            f(&mut config);
        }
        self.save().await
    }
}

/// Merge discovered agents with stored config. Keeps only agents in `discovered` (cleans up removed types).
/// Preserves `featured` from existing config, or migrates from `dashboard_featured` when existing is empty.
pub fn merge_coding_agents(
    existing: &[CodingAgent],
    discovered: Vec<CodingAgent>,
    dashboard_featured: &[String],
) -> Vec<CodingAgent> {
    let use_dashboard_migration =
        existing.is_empty() && !dashboard_featured.is_empty();
    discovered
        .into_iter()
        .map(|mut d| {
            let featured = existing
                .iter()
                .find(|e| e.agent_type == d.agent_type)
                .map(|e| e.featured)
                .unwrap_or_else(|| {
                    if use_dashboard_migration {
                        let key = agent_type_to_config_key(&d.agent_type);
                        dashboard_featured.iter().any(|s| s == &key)
                    } else {
                        true
                    }
                });
            d.featured = featured;
            d
        })
        .collect()
}

fn agent_type_to_config_key(agent_type: &AgentType) -> String {
    format!("{:?}", agent_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_store_init() {
        let temp_dir = tempdir().unwrap();
        let store = ConfigStore::new(temp_dir.path().to_path_buf());
        
        store.init().await.unwrap();
        
        let config = store.get_config().await;
        assert!(config.providers.is_empty());
        assert!(config.routing_rules.is_empty());
    }

    #[tokio::test]
    async fn test_config_store_save_load() {
        let temp_dir = tempdir().unwrap();
        let store = ConfigStore::new(temp_dir.path().to_path_buf());
        
        store.init().await.unwrap();
        
        // Update config
        store.update(|config| {
            config.app.enable_proxy = true;
        }).await.unwrap();
        
        // Reload and verify
        store.load().await.unwrap();
        let config = store.get_config().await;
        assert!(config.app.enable_proxy);
    }
}
