use std::collections::HashSet;
use std::sync::Arc;
use chrono::Utc;
use glob::Pattern;

use crate::models::{ApiGroup, CreateRuleInput, RoutingRule, RuleType, UpdateRuleInput};
use crate::storage::ConfigStore;

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("Rule not found: {0}")]
    RuleNotFound(String),
    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::StorageError),
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}

pub struct RouterService {
    store: Arc<ConfigStore>,
}

impl RouterService {
    pub fn new(store: Arc<ConfigStore>) -> Self {
        Self { store }
    }

    pub async fn list_rules(&self) -> Result<Vec<RoutingRule>, RouterError> {
        let config = self.store.get_config().await;
        let (mut rules, has_duplicates) = deduplicate_rules(config.routing_rules);

        if has_duplicates {
            // Persist cleaned rules so future fetches stay deduped
            let rules_to_save = rules.clone();
            self.store
                .update(|cfg| {
                    cfg.routing_rules = rules_to_save;
                })
                .await?;
        }

        rules.sort_by_key(|r| {
            (
                api_group_order(&r.api_group),
                rule_type_order(&r.rule_type),
                r.priority,
            )
        });
        Ok(rules)
    }

    pub async fn get_rule(&self, id: &str) -> Result<RoutingRule, RouterError> {
        let config = self.store.get_config().await;
        config
            .routing_rules
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| RouterError::RuleNotFound(id.to_string()))
    }

    pub async fn create_rule(&self, input: CreateRuleInput) -> Result<RoutingRule, RouterError> {
        // Validate the pattern
        Pattern::new(&input.match_pattern)
            .map_err(|_| RouterError::InvalidPattern(input.match_pattern.clone()))?;
        validate_api_group_pattern(&input.api_group, &input.rule_type, &input.match_pattern)?;

        let config = self.store.get_config().await;

        // Skip creating duplicate rules (same api group + rule type + pattern)
        if let Some(existing) = config
            .routing_rules
            .iter()
            .find(|r| {
                r.api_group == input.api_group
                    && r.rule_type == input.rule_type
                    && r.match_pattern == input.match_pattern
            })
        {
            return Ok(existing.clone());
        }

        let priority = config
            .routing_rules
            .iter()
            .filter(|r| r.api_group == input.api_group && r.rule_type == input.rule_type)
            .map(|r| r.priority)
            .max()
            .unwrap_or(0)
            + 1;

        let mut rule = RoutingRule::new(
            input.provider_id,
            input.match_pattern,
            priority,
            input.rule_type,
            input.api_group,
        );
        rule.model_rewrite = input.model_rewrite;
        rule.enabled = input.enabled;

        let rule_clone = rule.clone();
        self.store
            .update(|config| {
                config.routing_rules.push(rule_clone);
            })
            .await?;

        Ok(rule)
    }

    pub async fn update_rule(
        &self,
        id: &str,
        input: UpdateRuleInput,
    ) -> Result<RoutingRule, RouterError> {
        // First check if rule exists
        let existing = self.get_rule(id).await?;

        // Validate pattern if provided
        if let Some(ref pattern) = input.match_pattern {
            Pattern::new(pattern).map_err(|_| RouterError::InvalidPattern(pattern.clone()))?;
        }

        let next_api_group = input.api_group.clone().unwrap_or(existing.api_group);
        let next_rule_type = input.rule_type.clone().unwrap_or(existing.rule_type);
        let next_pattern = input.match_pattern.clone().unwrap_or(existing.match_pattern);
        validate_api_group_pattern(&next_api_group, &next_rule_type, &next_pattern)?;

        let id_owned = id.to_string();
        self.store
            .update(|config| {
                if let Some(rule) = config.routing_rules.iter_mut().find(|r| r.id == id_owned) {
                    if let Some(rule_type) = input.rule_type.clone() {
                        rule.rule_type = rule_type;
                    }
                    if let Some(api_group) = input.api_group.clone() {
                        rule.api_group = api_group;
                    }
                    if let Some(provider_id) = input.provider_id.clone() {
                        rule.provider_id = provider_id;
                    }
                    if let Some(match_pattern) = input.match_pattern.clone() {
                        rule.match_pattern = match_pattern;
                    }
                    if let Some(model_rewrite) = input.model_rewrite.clone() {
                        rule.model_rewrite = Some(model_rewrite);
                    }
                    if let Some(enabled) = input.enabled {
                        rule.enabled = enabled;
                    }
                    rule.updated_at = Utc::now();
                }
            })
            .await?;

        self.get_rule(id).await
    }

    pub async fn delete_rule(&self, id: &str) -> Result<(), RouterError> {
        // First check if rule exists
        self.get_rule(id).await?;

        let id_owned = id.to_string();
        self.store
            .update(|config| {
                config.routing_rules.retain(|r| r.id != id_owned);
            })
            .await?;

        Ok(())
    }

    pub async fn reorder_rules(&self, rule_ids: Vec<String>) -> Result<(), RouterError> {
        self.store
            .update(|config| {
                for (index, rule_id) in rule_ids.iter().enumerate() {
                    if let Some(rule) = config.routing_rules.iter_mut().find(|r| r.id == *rule_id) {
                        rule.priority = index as i32 + 1;
                        rule.updated_at = Utc::now();
                    }
                }
            })
            .await?;

        Ok(())
    }

    /// Match a model name against routing rules
    #[cfg(test)]
    pub fn matches_pattern(pattern: &str, model_name: &str) -> Result<bool, RouterError> {
        let pattern =
            Pattern::new(pattern).map_err(|_| RouterError::InvalidPattern(pattern.to_string()))?;
        Ok(pattern.matches(model_name))
    }
}

fn validate_api_group_pattern(
    api_group: &ApiGroup,
    rule_type: &RuleType,
    pattern: &str,
) -> Result<(), RouterError> {
    if *rule_type == RuleType::Path && *api_group == ApiGroup::Generic {
        if pattern.starts_with("/api/openai") || pattern.starts_with("/api/anthropic") {
            return Err(RouterError::InvalidPattern(pattern.to_string()));
        }
    }

    Ok(())
}

fn api_group_order(api_group: &ApiGroup) -> u8 {
    match api_group {
        ApiGroup::OpenAI => 0,
        ApiGroup::Anthropic => 1,
        ApiGroup::Generic => 2,
    }
}

fn rule_type_order(rule_type: &RuleType) -> u8 {
    match rule_type {
        RuleType::Path => 0,
        RuleType::Model => 1,
    }
}

fn deduplicate_rules(rules: Vec<RoutingRule>) -> (Vec<RoutingRule>, bool) {
    let original_len = rules.len();
    let mut seen = HashSet::new();
    let mut deduped = Vec::with_capacity(original_len);

    for rule in rules {
        let key = (
            rule.api_group.clone(),
            rule.rule_type.clone(),
            rule.match_pattern.clone(),
        );

        if seen.insert(key) {
            deduped.push(rule);
        }
    }

    let changed = deduped.len() != original_len;
    (deduped, changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matching() {
        // Exact match
        assert!(RouterService::matches_pattern("gpt-4", "gpt-4").unwrap());
        assert!(!RouterService::matches_pattern("gpt-4", "gpt-4-turbo").unwrap());

        // Wildcard match
        assert!(RouterService::matches_pattern("gpt-4*", "gpt-4").unwrap());
        assert!(RouterService::matches_pattern("gpt-4*", "gpt-4-turbo").unwrap());
        assert!(RouterService::matches_pattern("claude-*", "claude-3-5-sonnet").unwrap());

        // Complex patterns
        assert!(RouterService::matches_pattern("*-turbo", "gpt-4-turbo").unwrap());
    }
}
