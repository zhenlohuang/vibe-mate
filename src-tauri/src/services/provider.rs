use std::sync::Arc;
use chrono::Utc;

use crate::models::{
    AgentProviderType, ConnectionStatus, CreateProviderInput, Provider, ProviderCategory,
    ProviderStatus, ProviderType, UpdateProviderInput,
};
use crate::agents::auth::auth_path_for_provider_id;
use crate::storage::ConfigStore;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Provider not found: {0}")]
    NotFound(String),
    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::StorageError),
}

pub struct ProviderService {
    store: Arc<ConfigStore>,
}

impl ProviderService {
    pub fn new(store: Arc<ConfigStore>) -> Self {
        Self { store }
    }

    pub async fn list_providers(&self) -> Result<Vec<Provider>, ProviderError> {
        let config = self.store.get_config().await;
        let mut providers = config.providers;
        let mut status_updates: Vec<(String, ProviderStatus)> = Vec::new();
        let mut base_url_updates: Vec<(String, String)> = Vec::new();

        for provider in providers.iter_mut() {
            if provider.provider_category != ProviderCategory::Agent {
                continue;
            }
            if let ProviderType::Agent(agent_type) = &provider.provider_type {
                let current = provider.api_base_url.as_deref().unwrap_or("").trim();
                if current.is_empty() {
                    let default_url = default_agent_api_base_url(agent_type).to_string();
                    provider.api_base_url = Some(default_url.clone());
                    base_url_updates.push((provider.id.clone(), default_url));
                }
            }
            let is_logged_in = auth_path_for_provider_id(&provider.id)
                .map(|path| path.exists())
                .unwrap_or(false);
            let next_status = if is_logged_in {
                ProviderStatus::Connected
            } else {
                ProviderStatus::Disconnected
            };
            if provider.status != next_status {
                status_updates.push((provider.id.clone(), next_status.clone()));
                provider.status = next_status;
            }
        }

        if !status_updates.is_empty() || !base_url_updates.is_empty() {
            let now = Utc::now();
            self.store
                .update(|config| {
                    for (id, api_base_url) in &base_url_updates {
                        if let Some(provider) =
                            config.providers.iter_mut().find(|p| p.id == *id)
                        {
                            provider.api_base_url = Some(api_base_url.clone());
                            provider.updated_at = now;
                        }
                    }
                    for (id, status) in &status_updates {
                        if let Some(provider) =
                            config.providers.iter_mut().find(|p| p.id == *id)
                        {
                            provider.status = status.clone();
                            provider.updated_at = now;
                        }
                    }
                })
                .await?;
        }

        Ok(providers)
    }

    pub async fn get_provider(&self, id: &str) -> Result<Provider, ProviderError> {
        let config = self.store.get_config().await;
        config
            .providers
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| ProviderError::NotFound(id.to_string()))
    }

    pub async fn create_provider(
        &self,
        input: CreateProviderInput,
    ) -> Result<Provider, ProviderError> {
        let provider = match input.provider_category {
            ProviderCategory::Model => {
                if let ProviderType::Model(model_type) = input.provider_type {
                    Provider::new_model(
                        input.name,
                        model_type,
                        input.api_base_url.unwrap_or_default(),
                        input.api_key.unwrap_or_default(),
                    )
                } else {
                    return Err(ProviderError::Storage(crate::storage::StorageError::Io(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Invalid provider type for Model category",
                        ),
                    )));
                }
            }
            ProviderCategory::Agent => {
                if let ProviderType::Agent(agent_type) = input.provider_type {
                    let mut provider = Provider::new_agent(input.name, agent_type.clone());
                    let base_url = input
                        .api_base_url
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or_else(|| default_agent_api_base_url(&agent_type).to_string());
                    provider.api_base_url = Some(base_url);
                    provider
                } else {
                    return Err(ProviderError::Storage(crate::storage::StorageError::Io(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Invalid provider type for Agent category",
                        ),
                    )));
                }
            }
        };

        let provider_clone = provider.clone();
        self.store
            .update(|config| {
                config.providers.push(provider_clone.clone());
            })
            .await?;

        // Return the updated provider
        self.get_provider(&provider.id).await
    }

    pub async fn update_provider(
        &self,
        id: &str,
        input: UpdateProviderInput,
    ) -> Result<Provider, ProviderError> {
        // First check if provider exists
        self.get_provider(id).await?;

        let id_owned = id.to_string();
        self.store
            .update(|config| {
                if let Some(provider) = config.providers.iter_mut().find(|p| p.id == id_owned) {
                    if let Some(name) = input.name.clone() {
                        provider.name = name;
                    }
                    if input.api_base_url.is_some() {
                        provider.api_base_url = input.api_base_url.clone();
                    }
                    if input.api_key.is_some() {
                        provider.api_key = input.api_key.clone();
                    }
                    provider.updated_at = Utc::now();
                }
            })
            .await?;

        self.get_provider(id).await
    }

    pub async fn delete_provider(&self, id: &str) -> Result<(), ProviderError> {
        // First check if provider exists
        self.get_provider(id).await?;

        let id_owned = id.to_string();
        self.store
            .update(|config| {
                config.providers.retain(|p| p.id != id_owned);
                // Also remove routing rules that reference this provider
                config.routing_rules.retain(|r| r.provider_id != id_owned);
            })
            .await?;

        Ok(())
    }

    pub async fn set_default_provider(&self, id: &str) -> Result<(), ProviderError> {
        // First check if provider exists
        self.get_provider(id).await?;

        let id_owned = id.to_string();
        self.store
            .update(|config| {
                if let Some(index) = config.providers.iter().position(|p| p.id == id_owned) {
                    let mut provider = config.providers.remove(index);
                    provider.updated_at = Utc::now();
                    config.providers.insert(0, provider);
                }
            })
            .await?;

        Ok(())
    }

    pub async fn test_connection(&self, id: &str) -> Result<ConnectionStatus, ProviderError> {
        let provider = self.get_provider(id).await?;

        // Simple connectivity test - just check if the URL is reachable
        // In a real implementation, you'd make an actual API call
        let start = std::time::Instant::now();

        // For now, we'll simulate a successful connection
        // In production, you'd use reqwest to actually test the endpoint
        let is_connected = match provider.provider_category {
            ProviderCategory::Agent => auth_path_for_provider_id(&provider.id)
                .map(|path| path.exists())
                .unwrap_or(false),
            ProviderCategory::Model => provider.api_key.as_ref().map_or(false, |k| !k.is_empty())
                && provider.api_base_url.as_ref().map_or(false, |u| !u.is_empty()),
        };
        let latency_ms = start.elapsed().as_millis() as u64;

        // Update provider status
        let id_owned = id.to_string();
        let status = if is_connected {
            ProviderStatus::Connected
        } else {
            ProviderStatus::Disconnected
        };
        let status_clone = status.clone();

        self.store
            .update(|config| {
                if let Some(provider) = config.providers.iter_mut().find(|p| p.id == id_owned) {
                    provider.status = status_clone;
                    provider.updated_at = Utc::now();
                }
            })
            .await?;

        Ok(ConnectionStatus {
            is_connected,
            latency_ms: Some(latency_ms),
            error: if is_connected {
                None
            } else {
                Some("Invalid configuration".to_string())
            },
        })
    }
}

fn default_agent_api_base_url(agent_type: &AgentProviderType) -> &'static str {
    match agent_type {
        AgentProviderType::Codex => "https://chatgpt.com/backend-api/codex",
        AgentProviderType::ClaudeCode => "https://api.anthropic.com",
        AgentProviderType::GeminiCli => "https://cloudcode-pa.googleapis.com",
        AgentProviderType::Antigravity => "https://cloudcode-pa.googleapis.com",
    }
}
