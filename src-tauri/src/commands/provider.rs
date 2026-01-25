use std::sync::Arc;
use tauri::State;

use crate::models::{
    AgentAuthStart, AgentQuota, ConnectionStatus, CreateProviderInput, Provider, UpdateProviderInput,
};
use crate::services::{AgentAuthService, ProviderService};

#[tauri::command]
pub async fn list_providers(
    service: State<'_, Arc<ProviderService>>,
) -> Result<Vec<Provider>, String> {
    service
        .list_providers()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_provider(
    service: State<'_, Arc<ProviderService>>,
    input: CreateProviderInput,
) -> Result<Provider, String> {
    service
        .create_provider(input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_provider(
    service: State<'_, Arc<ProviderService>>,
    id: String,
    input: UpdateProviderInput,
) -> Result<Provider, String> {
    service
        .update_provider(&id, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_provider(
    service: State<'_, Arc<ProviderService>>,
    id: String,
) -> Result<(), String> {
    service
        .delete_provider(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_default_provider(
    service: State<'_, Arc<ProviderService>>,
    id: String,
) -> Result<(), String> {
    service
        .set_default_provider(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_connection(
    service: State<'_, Arc<ProviderService>>,
    id: String,
) -> Result<ConnectionStatus, String> {
    service
        .test_connection(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_agent_auth(
    service: State<'_, Arc<AgentAuthService>>,
    provider_id: String,
) -> Result<AgentAuthStart, String> {
    service
        .start_auth(&provider_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_agent_auth(
    service: State<'_, Arc<AgentAuthService>>,
    flow_id: String,
) -> Result<Provider, String> {
    service
        .complete_auth(&flow_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agent_quota(
    service: State<'_, Arc<AgentAuthService>>,
    provider_id: String,
) -> Result<AgentQuota, String> {
    service
        .get_quota(&provider_id)
        .await
        .map_err(|e| e.to_string())
}
