use std::sync::Arc;

use tauri::State;

use crate::models::AgentType;
use crate::services::AgentProxyService;

#[tauri::command]
pub async fn is_agent_proxy_enabled(
    service: State<'_, Arc<AgentProxyService>>,
    agent_type: AgentType,
) -> Result<bool, String> {
    service
        .is_proxy_enabled(&agent_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_agent_proxy_enabled(
    service: State<'_, Arc<AgentProxyService>>,
    agent_type: AgentType,
    enabled: bool,
) -> Result<(), String> {
    service
        .set_proxy_enabled(&agent_type, enabled)
        .await
        .map_err(|e| e.to_string())
}
