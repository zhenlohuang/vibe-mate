use std::sync::Arc;
use chrono::Utc;

use crate::models::{
    ConnectionStatus, CreateProviderInput, Provider, ProviderStatus, UpdateProviderInput,
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
        let provider = Provider::new_model(
            input.name,
            input.provider_type,
            input.api_base_url.unwrap_or_default(),
            input.api_key.unwrap_or_default(),
        );

        let provider_clone = provider.clone();
        self.store
            .update(|config| {
                config.providers.push(provider_clone.clone());
            })
            .await?;

        self.get_provider(&provider.id).await
    }

    pub async fn update_provider(
        &self,
        id: &str,
        input: UpdateProviderInput,
    ) -> Result<Provider, ProviderError> {
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
        self.get_provider(id).await?;

        let id_owned = id.to_string();
        self.store
            .update(|config| {
                config.providers.retain(|p| p.id != id_owned);
                config.routing_rules.retain(|r| r.provider_id != id_owned);
            })
            .await?;

        Ok(())
    }

    pub async fn set_default_provider(&self, id: &str) -> Result<(), ProviderError> {
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
        let start = std::time::Instant::now();

        let is_connected = provider.api_key.as_ref().map_or(false, |k| !k.is_empty())
            && provider.api_base_url.as_ref().map_or(false, |u| !u.is_empty());
        let latency_ms = start.elapsed().as_millis() as u64;

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
