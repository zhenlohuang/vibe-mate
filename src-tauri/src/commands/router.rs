use std::sync::Arc;
use tauri::State;

use crate::models::{CreateRuleInput, RoutingRule, UpdateRuleInput};
use crate::services::RouterService;

#[tauri::command]
pub async fn list_rules(
    service: State<'_, Arc<RouterService>>,
) -> Result<Vec<RoutingRule>, String> {
    service
        .list_rules()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_rule(
    service: State<'_, Arc<RouterService>>,
    input: CreateRuleInput,
) -> Result<RoutingRule, String> {
    service
        .create_rule(input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_rule(
    service: State<'_, Arc<RouterService>>,
    id: String,
    input: UpdateRuleInput,
) -> Result<RoutingRule, String> {
    service
        .update_rule(&id, input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_rule(
    service: State<'_, Arc<RouterService>>,
    id: String,
) -> Result<(), String> {
    service
        .delete_rule(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reorder_rules(
    service: State<'_, Arc<RouterService>>,
    rule_ids: Vec<String>,
) -> Result<(), String> {
    service
        .reorder_rules(rule_ids)
        .await
        .map_err(|e| e.to_string())
}

