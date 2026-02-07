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
pub async fn read_agent_config(
    service: State<'_, Arc<AgentService>>,
    agent_type: AgentType,
    config_path: Option<String>,
) -> Result<String, String> {
    service
        .read_config(&agent_type, config_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_agent_config(
    service: State<'_, Arc<AgentService>>,
    agent_type: AgentType,
    content: String,
    config_path: Option<String>,
) -> Result<(), String> {
    service
        .save_config(&agent_type, content, config_path)
        .await
        .map_err(|e| e.to_string())
}
