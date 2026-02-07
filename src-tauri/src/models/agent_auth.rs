use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentProviderType {
    Codex,
    ClaudeCode,
    GeminiCli,
    Antigravity,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAuthStart {
    pub flow_id: String,
    pub auth_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentQuotaEntry {
    pub label: String,
    pub used_percent: f64,
    pub reset_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentQuota {
    pub plan_type: Option<String>,
    pub limit_reached: Option<bool>,
    pub session_used_percent: f64,
    pub session_reset_at: Option<i64>,
    pub week_used_percent: f64,
    pub week_reset_at: Option<i64>,
    pub entries: Option<Vec<AgentQuotaEntry>>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAccountInfo {
    pub agent_type: AgentProviderType,
    pub is_authenticated: bool,
    pub email: Option<String>,
}
