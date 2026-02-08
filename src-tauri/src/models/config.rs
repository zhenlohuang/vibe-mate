use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{CodingAgent, Provider, RoutingRule};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct AppConfig {
    /// Proxy server listen port (config key: app.port)
    pub port: u16,
    pub enable_proxy: bool,
    pub proxy_url: Option<String>,
    pub no_proxy: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 12345,
            enable_proxy: false,
            proxy_url: None,
            no_proxy: Vec::new(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAppConfigInput {
    pub port: Option<u16>,
    pub enable_proxy: Option<bool>,
    pub proxy_url: Option<String>,
    pub no_proxy: Option<Vec<String>>,
}

/// Unified configuration file structure (~/.vibemate/settings.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct VibeMateConfig {
    pub app: AppConfig,
    pub providers: Vec<Provider>,
    pub routing_rules: Vec<RoutingRule>,
    /// Persisted list of coding agents (discovered at startup); each has a `featured` flag.
    pub coding_agents: Vec<CodingAgent>,
}

impl Default for VibeMateConfig {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            providers: Vec::new(),
            routing_rules: Vec::new(),
            coding_agents: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyStatus {
    pub is_running: bool,
    pub port: u16,
    pub request_count: u64,
}

impl Default for ProxyStatus {
    fn default() -> Self {
        Self {
            is_running: false,
            port: 12345,
            request_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatencyResult {
    pub success: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}
