use std::sync::Arc;
use chrono::Utc;

use crate::models::{
    ConnectionStatus, CreateProviderInput, Provider, ProviderCategory, ProviderStatus,
    ProviderType, UpdateProviderInput,
};
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
        Ok(config.providers)
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
                    Provider::new_agent(input.name, agent_type, input.auth_path)
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
                // If this is the first provider, make it default
                let is_first = config.providers.is_empty();
                let mut new_provider = provider_clone.clone();
                new_provider.is_default = is_first;
                config.providers.push(new_provider);
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
                    if input.auth_path.is_some() {
                        provider.auth_path = input.auth_path.clone();
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
                for provider in config.providers.iter_mut() {
                    provider.is_default = provider.id == id_owned;
                    provider.updated_at = Utc::now();
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
        let is_connected = provider.api_key.as_ref().map_or(false, |k| !k.is_empty())
            && provider.api_base_url.as_ref().map_or(false, |u| !u.is_empty());
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

