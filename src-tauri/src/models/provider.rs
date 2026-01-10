use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Google,
    Azure,
    Custom,
}

impl Default for ProviderType {
    fn default() -> Self {
        Self::OpenAI
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
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub api_base_url: String,
    pub api_key: String,
    pub is_default: bool,
    pub enable_proxy: bool,
    pub status: ProviderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Provider {
    pub fn new(
        name: String,
        provider_type: ProviderType,
        api_base_url: String,
        api_key: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            provider_type,
            api_base_url,
            api_key,
            is_default: false,
            enable_proxy: true,
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
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub api_base_url: String,
    pub api_key: String,
    #[serde(default = "default_enable_proxy")]
    pub enable_proxy: bool,
}

fn default_enable_proxy() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderInput {
    pub name: Option<String>,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub enable_proxy: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub is_connected: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

