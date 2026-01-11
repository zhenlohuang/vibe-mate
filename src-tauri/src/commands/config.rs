use std::sync::Arc;
use tauri::State;

use crate::models::{AgentConfigItem, AppConfig, LatencyResult, UpdateAgentsConfigInput, UpdateAppConfigInput};
use crate::services::ConfigService;

#[tauri::command]
pub async fn get_config(
    service: State<'_, Arc<ConfigService>>,
) -> Result<AppConfig, String> {
    service
        .get_config()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_config(
    service: State<'_, Arc<ConfigService>>,
    input: UpdateAppConfigInput,
) -> Result<AppConfig, String> {
    service
        .update_config(input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_agents_config(
    service: State<'_, Arc<ConfigService>>,
) -> Result<Vec<AgentConfigItem>, String> {
    service
        .get_agents_config()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_agents_config(
    service: State<'_, Arc<ConfigService>>,
    input: UpdateAgentsConfigInput,
) -> Result<Vec<AgentConfigItem>, String> {
    service
        .update_agents_config(input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_latency(
    service: State<'_, Arc<ConfigService>>,
) -> Result<LatencyResult, String> {
    Ok(service.test_latency().await)
}
