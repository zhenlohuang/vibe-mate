use std::sync::Arc;
use chrono::Utc;

use crate::models::{AppConfig, LatencyResult, UpdateAppConfigInput};
use crate::storage::ConfigStore;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::StorageError),
}

pub struct ConfigService {
    store: Arc<ConfigStore>,
}

impl ConfigService {
    pub fn new(store: Arc<ConfigStore>) -> Self {
        Self { store }
    }

    pub async fn get_config(&self) -> Result<AppConfig, ConfigError> {
        let config = self.store.get_config().await;
        Ok(config.app)
    }

    pub async fn update_config(&self, input: UpdateAppConfigInput) -> Result<AppConfig, ConfigError> {
        self.store
            .update(|config| {
                if let Some(enable_proxy) = input.enable_proxy {
                    config.app.enable_proxy = enable_proxy;
                }
                if let Some(proxy_url) = input.proxy_url.clone() {
                    config.app.proxy_url = Some(proxy_url);
                }
                if let Some(no_proxy) = input.no_proxy.clone() {
                    config.app.no_proxy = no_proxy;
                }
                config.app.updated_at = Utc::now();
            })
            .await?;

        self.get_config().await
    }

    pub async fn test_latency(&self) -> LatencyResult {
        let config = self.store.get_config().await;
        
        // Test connectivity based on proxy settings
        let start = std::time::Instant::now();
        
        // For now, we'll just simulate a latency test
        // In production, you'd actually test network connectivity
        let success = if config.app.enable_proxy {
            config.app.proxy_url.is_some()
        } else {
            true
        };

        let latency_ms = start.elapsed().as_millis() as u64;

        LatencyResult {
            success,
            latency_ms: if success { Some(latency_ms) } else { None },
            error: if success {
                None
            } else {
                Some("Proxy configuration is incomplete".to_string())
            },
        }
    }
}
