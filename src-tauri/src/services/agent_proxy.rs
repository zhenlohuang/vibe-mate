use std::sync::Arc;

use bytes::Bytes;
use reqwest::Client;

use crate::agents::{get_agent_auth, AgentAuth, AgentAuthContext, AgentAuthError};
use crate::models::{Provider, ProviderCategory, ProviderType};
use crate::storage::ConfigStore;

/// Agent proxy error types
#[derive(Debug, thiserror::Error)]
pub enum AgentProxyError {
    #[error("Token not found for provider. Please login first.")]
    TokenNotFound,

    #[error("Failed to refresh token: {0}")]
    RefreshFailed(String),

    #[error("Unsupported agent type: {0}")]
    UnsupportedAgent(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Auth error: {0}")]
    Auth(#[from] AgentAuthError),
}

/// Service for handling Agent Provider proxy operations
pub struct AgentProxyService {
    store: Arc<ConfigStore>,
}

impl AgentProxyService {
    pub fn new(store: Arc<ConfigStore>) -> Self {
        Self { store }
    }

    /// Get valid authentication for an agent provider, refreshing token if necessary
    pub async fn get_agent_auth(
        &self,
        provider: &Provider,
    ) -> Result<AgentAuth, AgentProxyError> {
        // Validate provider is an Agent
        let agent_type = match &provider.provider_type {
            ProviderType::Agent(t) => t,
            _ => {
                return Err(AgentProxyError::UnsupportedAgent(
                    "Not an agent provider".into(),
                ))
            }
        };

        if provider.provider_category != ProviderCategory::Agent {
            return Err(AgentProxyError::UnsupportedAgent(
                "Provider category is not Agent".into(),
            ));
        }

        // Use the centralized agent auth function from agents module
        let ctx = AgentAuthContext::new(self.store.clone());
        let auth = get_agent_auth(&ctx, provider, agent_type).await?;

        Ok(auth)
    }

    /// Build the target URL for an agent request
    pub fn build_agent_url(
        &self,
        agent_auth: &AgentAuth,
        original_path: &str,
        api_group_prefix: &str,
    ) -> String {
        // Strip the API group prefix from the path
        let path = original_path
            .strip_prefix(api_group_prefix)
            .unwrap_or(original_path);

        let base_url = agent_auth.api_base_url.trim_end_matches('/');

        // Handle /v1 duplication for OpenAI-compatible APIs
        if base_url.ends_with("/v1") && path.starts_with("/v1") {
            format!("{}{}", base_url, &path[3..])
        } else {
            format!("{}{}", base_url, path)
        }
    }

    /// Execute a request to an agent provider
    pub async fn execute_request(
        &self,
        http_client: &Client,
        agent_auth: &AgentAuth,
        method: reqwest::Method,
        target_url: &str,
        body: Vec<u8>,
        original_headers: &[(String, String)],
    ) -> Result<reqwest::Response, AgentProxyError> {
        let mut request = http_client.request(method, target_url);

        // Copy original headers (excluding certain headers)
        const SKIPPED_HEADERS: &[&str] = &[
            "host",
            "authorization",
            "proxy-authorization",
            "content-length",
            "transfer-encoding",
            "connection",
        ];

        for (key, value) in original_headers {
            if SKIPPED_HEADERS
                .iter()
                .any(|header| key.eq_ignore_ascii_case(header))
            {
                continue;
            }

            request = request.header(key, value);
        }

        // Add authentication header
        request = request.header("Authorization", format!("Bearer {}", agent_auth.access_token));

        // Add additional headers specific to the agent
        for (key, value) in &agent_auth.additional_headers {
            request = request.header(key, value);
        }

        // Set content type and body
        request = request
            .header("Content-Type", "application/json")
            .body(body);

        let response = request.send().await?;
        Ok(response)
    }
}

/// Transform request body for Claude Code OAuth tokens (add proxy_ prefix to tool names)
pub fn transform_claude_oauth_request(body: &Bytes) -> Vec<u8> {
    if let Ok(mut json) = serde_json::from_slice::<serde_json::Value>(body) {
        add_tool_prefix(&mut json, "proxy_");
        serde_json::to_vec(&json).unwrap_or_else(|_| body.to_vec())
    } else {
        body.to_vec()
    }
}

/// Add prefix to tool names in request body
fn add_tool_prefix(json: &mut serde_json::Value, prefix: &str) {
    // Add prefix to tools array
    if let Some(tools) = json.get_mut("tools").and_then(|t| t.as_array_mut()) {
        for tool in tools {
            // First get the name as owned String
            let name = tool
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());

            if let Some(name) = name {
                if let Some(obj) = tool.as_object_mut() {
                    obj.insert(
                        "name".to_string(),
                        serde_json::Value::String(format!("{}{}", prefix, name)),
                    );
                }
            }
        }
    }

    // Add prefix to tool_choice if it's a specific tool
    if let Some(tool_choice) = json.get_mut("tool_choice") {
        if let Some(obj) = tool_choice.as_object_mut() {
            // First get the name as owned String
            let name = obj.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());

            if let Some(name) = name {
                obj.insert(
                    "name".to_string(),
                    serde_json::Value::String(format!("{}{}", prefix, name)),
                );
            }
        }
    }
}
