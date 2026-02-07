use std::sync::Arc;
use tauri::State;

use crate::models::{AgentAccountInfo, AgentAuthStart, AgentQuota, AgentProviderType};
use crate::services::AgentAuthService;

#[tauri::command]
pub async fn start_agent_auth(
    service: State<'_, Arc<AgentAuthService>>,
    agent_type: AgentProviderType,
) -> Result<AgentAuthStart, String> {
    service
        .start_auth(agent_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn complete_agent_auth(
    service: State<'_, Arc<AgentAuthService>>,
    flow_id: String,
) -> Result<AgentAccountInfo, String> {
    service
        .complete_auth(&flow_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agent_quota(
    service: State<'_, Arc<AgentAuthService>>,
    agent_type: AgentProviderType,
) -> Result<AgentQuota, String> {
    service
        .get_quota(agent_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_agent_accounts(
    service: State<'_, Arc<AgentAuthService>>,
) -> Result<Vec<AgentAccountInfo>, String> {
    Ok(service.list_accounts().await)
}

#[tauri::command]
pub async fn remove_agent_auth(
    service: State<'_, Arc<AgentAuthService>>,
    agent_type: AgentProviderType,
) -> Result<(), String> {
    service
        .remove_auth(&agent_type)
        .await
        .map_err(|e| e.to_string())
}
