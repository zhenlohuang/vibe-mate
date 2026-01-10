use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RuleType {
    #[serde(rename = "path")]
    Path,
    #[serde(rename = "model")]
    Model,
}

impl Default for RuleType {
    fn default() -> Self {
        Self::Model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ApiGroup {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "generic")]
    Generic,
}

impl Default for ApiGroup {
    fn default() -> Self {
        Self::Generic
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingRule {
    pub id: String,
    #[serde(default)]
    pub rule_type: RuleType,
    #[serde(default)]
    pub api_group: ApiGroup,
    pub provider_id: String,
    pub match_pattern: String,
    pub model_rewrite: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RoutingRule {
    pub fn new(
        provider_id: String,
        match_pattern: String,
        priority: i32,
        rule_type: RuleType,
        api_group: ApiGroup,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            rule_type,
            api_group,
            provider_id,
            match_pattern,
            model_rewrite: None,
            priority,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleInput {
    #[serde(default)]
    pub rule_type: RuleType,
    #[serde(default)]
    pub api_group: ApiGroup,
    pub provider_id: String,
    pub match_pattern: String,
    pub model_rewrite: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRuleInput {
    pub rule_type: Option<RuleType>,
    pub api_group: Option<ApiGroup>,
    pub provider_id: Option<String>,
    pub match_pattern: Option<String>,
    pub model_rewrite: Option<String>,
    pub enabled: Option<bool>,
}
