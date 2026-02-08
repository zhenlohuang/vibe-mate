use std::sync::Arc;
use tauri::State;

use crate::models::{AppConfig, CodingAgent, LatencyResult, UpdateAppConfigInput};
use crate::services::{AgentService, ConfigService};
use crate::storage::{merge_coding_agents, ConfigStore};
use crate::models::AgentType;

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
pub async fn test_latency(
    service: State<'_, Arc<ConfigService>>,
) -> Result<LatencyResult, String> {
    Ok(service.test_latency().await)
}

#[tauri::command]
pub async fn get_coding_agents(
    store: State<'_, Arc<ConfigStore>>,
) -> Result<Vec<CodingAgent>, String> {
    let config = store.get_config().await;
    Ok(config.coding_agents)
}

#[tauri::command]
pub async fn refresh_coding_agents(
    store: State<'_, Arc<ConfigStore>>,
    agent_service: State<'_, Arc<AgentService>>,
) -> Result<Vec<CodingAgent>, String> {
    let discovered = agent_service
        .discover_agents()
        .map_err(|e| e.to_string())?;
    let config = store.get_config().await;
    let merged = merge_coding_agents(
        &config.coding_agents,
        discovered,
    );
    store
        .update(|c| c.coding_agents = merged.clone())
        .await
        .map_err(|e| e.to_string())?;
    Ok(merged)
}

#[tauri::command]
pub async fn set_coding_agent_featured(
    store: State<'_, Arc<ConfigStore>>,
    agent_type: AgentType,
    featured: bool,
) -> Result<Vec<CodingAgent>, String> {
    store
        .update(|config| {
            if let Some(entry) = config
                .coding_agents
                .iter_mut()
                .find(|a| a.agent_type == agent_type)
            {
                entry.featured = featured;
            }
        })
        .await
        .map_err(|e| e.to_string())?;
    let config = store.get_config().await;
    Ok(config.coding_agents)
}
