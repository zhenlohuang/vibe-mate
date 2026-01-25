use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAuthStart {
    pub flow_id: String,
    pub auth_url: String,
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
}
