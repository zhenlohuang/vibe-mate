use std::sync::Arc;
use chrono::Utc;

use crate::models::{AgentConfigItem, AppConfig, LatencyResult, UpdateAgentsConfigInput, UpdateAppConfigInput};
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
                if let Some(proxy_mode) = input.proxy_mode.clone() {
                    config.app.proxy_mode = proxy_mode;
                }
                if let Some(proxy_host) = input.proxy_host.clone() {
                    config.app.proxy_host = Some(proxy_host);
                }
                if let Some(proxy_port) = input.proxy_port {
                    config.app.proxy_port = Some(proxy_port);
                }
                if let Some(proxy_server_port) = input.proxy_server_port {
                    config.app.proxy_server_port = proxy_server_port;
                }
                if let Some(theme) = input.theme.clone() {
                    config.app.theme = theme;
                }
                if let Some(language) = input.language.clone() {
                    config.app.language = language;
                }
                config.app.updated_at = Utc::now();
            })
            .await?;

        self.get_config().await
    }

    pub async fn get_agents_config(&self) -> Result<Vec<AgentConfigItem>, ConfigError> {
        let config = self.store.get_config().await;
        Ok(config.agents)
    }

    pub async fn update_agents_config(
        &self,
        input: UpdateAgentsConfigInput,
    ) -> Result<Vec<AgentConfigItem>, ConfigError> {
        self.store
            .update(|config| {
                if let Some(agents) = input.agents.clone() {
                    config.agents = agents;
                }
            })
            .await?;

        self.get_agents_config().await
    }

    pub async fn test_latency(&self) -> LatencyResult {
        let config = self.store.get_config().await;
        
        // Test connectivity based on proxy settings
        let start = std::time::Instant::now();
        
        // For now, we'll just simulate a latency test
        // In production, you'd actually test network connectivity
        let success = match config.app.proxy_mode {
            crate::models::ProxyMode::None => true,
            crate::models::ProxyMode::System => true, // Assume system proxy works
            crate::models::ProxyMode::Custom => {
                config.app.proxy_host.is_some() && config.app.proxy_port.is_some()
            }
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
