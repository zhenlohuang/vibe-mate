use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

use crate::models::{CodingAgent, VibeMateConfig};

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

    /// Load configuration from file
    pub async fn load(&self) -> Result<(), StorageError> {
        let path = self.config_path();
        let config = if path.exists() {
            let content = fs::read_to_string(&path).await?;
            serde_json::from_str::<VibeMateConfig>(&content)
                .unwrap_or_default()
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
/// Preserves user-managed fields (`featured`, `proxy_enabled`) from existing config.
pub fn merge_coding_agents(
    existing: &[CodingAgent],
    discovered: Vec<CodingAgent>,
) -> Vec<CodingAgent> {
    discovered
        .into_iter()
        .map(|mut d| {
            if let Some(existing_entry) = existing.iter().find(|e| e.agent_type == d.agent_type) {
                d.featured = existing_entry.featured;
                d.proxy_enabled = existing_entry.proxy_enabled;
            }
            d
        })
        .collect()
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
