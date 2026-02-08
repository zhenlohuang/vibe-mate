use std::sync::Arc;
use tauri::State;

use crate::models::{AgentType, CodingAgent};
use crate::services::AgentService;

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
