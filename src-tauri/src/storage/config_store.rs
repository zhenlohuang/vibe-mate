use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

use crate::models::VibeMateConfig;

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

    /// Migrate legacy proxy settings (proxyHost + proxyPort) to proxyUrl
    fn migrate_proxy_config(raw: &mut serde_json::Value) {
        if let Some(app) = raw.get_mut("app") {
            let has_proxy_url = app.get("proxyUrl").and_then(|v| v.as_str()).is_some();
            if !has_proxy_url {
                let host = app.get("proxyHost").and_then(|v| v.as_str()).map(String::from);
                let port = app.get("proxyPort").and_then(|v| v.as_u64());
                if let (Some(h), Some(p)) = (host, port) {
                    if !h.is_empty() {
                        let proxy_url = format!("http://{}:{}", h, p);
                        if let Some(obj) = app.as_object_mut() {
                            obj.insert("proxyUrl".to_string(), serde_json::Value::String(proxy_url));
                            obj.remove("proxyHost");
                            obj.remove("proxyPort");
                        }
                    }
                }
            }
        }
    }

    /// Load configuration from file. Migrates legacy config (e.g. drops Agent providers, proxy host/port).
    pub async fn load(&self) -> Result<(), StorageError> {
        let path = self.config_path();
        let config = if path.exists() {
            let content = fs::read_to_string(&path).await?;
            let mut raw: serde_json::Value = serde_json::from_str(&content)?;

            // Apply migrations on raw JSON before deserializing
            Self::migrate_proxy_config(&mut raw);

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
                    VibeMateConfig {
                        app,
                        providers,
                        routing_rules,
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
