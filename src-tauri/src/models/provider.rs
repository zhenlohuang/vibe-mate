use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderCategory {
    Model,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentProviderType {
    Codex,
    ClaudeCode,
    GeminiCli,
    Antigravity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelProviderType {
    OpenAI,
    Anthropic,
    Google,
    OpenRouter,
    Custom,
}

impl Default for ModelProviderType {
    fn default() -> Self {
        Self::OpenAI
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ProviderType {
    Model(ModelProviderType),
    Agent(AgentProviderType),
}

impl Default for ProviderType {
    fn default() -> Self {
        Self::Model(ModelProviderType::default())
    }
}

impl ProviderType {
    #[allow(dead_code)]
    pub fn is_model(&self) -> bool {
        matches!(self, ProviderType::Model(_))
    }

    #[allow(dead_code)]
    pub fn is_agent(&self) -> bool {
        matches!(self, ProviderType::Agent(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderStatus {
    Connected,
    Disconnected,
    Error,
}

impl Default for ProviderStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    #[serde(rename = "category")]
    pub provider_category: ProviderCategory,
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub auth_path: Option<String>,
    pub auth_email: Option<String>,
    pub is_default: bool,
    pub status: ProviderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Provider {
    pub fn new_model(
        name: String,
        provider_type: ModelProviderType,
        api_base_url: String,
        api_key: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            provider_category: ProviderCategory::Model,
            provider_type: ProviderType::Model(provider_type),
            api_base_url: Some(api_base_url),
            api_key: Some(api_key),
            auth_path: None,
            auth_email: None,
            is_default: false,
            status: ProviderStatus::Disconnected,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_agent(
        name: String,
        provider_type: AgentProviderType,
        auth_path: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            provider_category: ProviderCategory::Agent,
            provider_type: ProviderType::Agent(provider_type),
            api_base_url: None,
            api_key: None,
            auth_path,
            auth_email: None,
            is_default: false,
            status: ProviderStatus::Disconnected,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProviderInput {
    pub name: String,
    #[serde(rename = "category")]
    pub provider_category: ProviderCategory,
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub auth_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderInput {
    pub name: Option<String>,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub auth_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub is_connected: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}
