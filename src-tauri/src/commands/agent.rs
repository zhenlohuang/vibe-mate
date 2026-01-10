use std::sync::Arc;
use tauri::State;

use crate::models::{AgentType, CodingAgent};
use crate::services::AgentService;

#[tauri::command]
pub async fn discover_agents(
    service: State<'_, Arc<AgentService>>,
) -> Result<Vec<CodingAgent>, String> {
    service
        .discover_agents()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_status(
    service: State<'_, Arc<AgentService>>,
    agent_type: AgentType,
) -> Result<CodingAgent, String> {
    service
        .check_status(&agent_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_login(
    service: State<'_, Arc<AgentService>>,
    agent_type: AgentType,
) -> Result<(), String> {
    service
        .open_login(&agent_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agent_version(
    service: State<'_, Arc<AgentService>>,
    agent_type: AgentType,
) -> Result<Option<String>, String> {
    Ok(service.get_version(&agent_type).await)
}

