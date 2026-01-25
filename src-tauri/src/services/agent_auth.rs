use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode as AxumStatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{NoProxy, Proxy, StatusCode as ReqwestStatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;
use tracing::{debug, info, warn};

use crate::models::{
    AgentAuthStart, AgentProviderType, AgentQuota, AgentQuotaEntry, Provider, ProviderStatus,
    ProviderType,
};
use crate::storage::ConfigStore;

const OPENAI_AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const OPENAI_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const OPENAI_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const CODEX_REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const CODEX_CALLBACK_PATH: &str = "/auth/callback";
const CODEX_CALLBACK_PORT: u16 = 1455;
const ORIGINATOR: &str = "codex_cli_rs";
const CODEX_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

const ANTHROPIC_AUTH_URL: &str = "https://claude.ai/oauth/authorize";
const ANTHROPIC_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const ANTHROPIC_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const CLAUDE_REDIRECT_URI: &str = "http://localhost:54545/callback";
const CLAUDE_CALLBACK_PATH: &str = "/callback";
const CLAUDE_CALLBACK_PORT: u16 = 54545;
const CLAUDE_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v1/userinfo?alt=json";

const ANTIGRAVITY_CLIENT_ID: &str =
    "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
const ANTIGRAVITY_CLIENT_SECRET: &str = "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";
const ANTIGRAVITY_REDIRECT_URI: &str = "http://localhost:51121/oauth-callback";
const ANTIGRAVITY_CALLBACK_PATH: &str = "/oauth-callback";
const ANTIGRAVITY_CALLBACK_PORT: u16 = 51121;
const ANTIGRAVITY_FETCH_MODELS_URL: &str =
    "https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels";
const ANTIGRAVITY_LOAD_CODE_ASSIST_URL: &str =
    "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
const ANTIGRAVITY_ONBOARD_USER_URL: &str =
    "https://cloudcode-pa.googleapis.com/v1internal:onboardUser";

const GEMINI_CLIENT_ID: &str =
    "681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com";
const GEMINI_CLIENT_SECRET: &str = "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";
const GEMINI_REDIRECT_URI: &str = "http://localhost:8085/oauth2callback";
const GEMINI_CALLBACK_PATH: &str = "/oauth2callback";
const GEMINI_CALLBACK_PORT: u16 = 8085;

const CODEX_SCOPES: &[&str] = &["openid", "email", "profile", "offline_access"];
const CLAUDE_SCOPES: &[&str] = &["org:create_api_key", "user:profile", "user:inference"];
const GEMINI_SCOPES: &[&str] = &[
    "openid",
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
];
const ANTIGRAVITY_SCOPES: &[&str] = &[
    "openid",
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
    "https://www.googleapis.com/auth/cclog",
    "https://www.googleapis.com/auth/experimentsandconfigs",
];

#[derive(Debug, thiserror::Error)]
pub enum AgentAuthError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    #[error("Provider is not an agent: {0}")]
    NotAgentProvider(String),
    #[error("Agent provider not supported: {0}")]
    UnsupportedAgentProvider(String),
    #[error("Auth flow already in progress")]
    FlowInProgress,
    #[error("Auth flow not found: {0}")]
    FlowNotFound(String),
    #[error("Auth flow timed out")]
    Timeout,
    #[error("Invalid auth callback: {0}")]
    InvalidCallback(String),
    #[error("Unauthorized - token expired or invalid")]
    Unauthorized,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::StorageError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodexTokenStorage {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub account_id: String,
    pub email: String,
    pub last_refresh: String,
    pub expire: String,
}

#[derive(Debug, Deserialize)]
struct CodexTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    expires_in: i64,
}

#[derive(Debug, Deserialize)]
struct CodexUsageResponse {
    plan_type: Option<String>,
    rate_limit: CodexRateLimit,
}

#[derive(Debug, Deserialize)]
struct CodexRateLimit {
    limit_reached: bool,
    primary_window: CodexUsageWindow,
    secondary_window: CodexUsageWindow,
}

#[derive(Debug, Deserialize)]
struct CodexUsageWindow {
    used_percent: f64,
    reset_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeTokenStorage {
    pub access_token: String,
    pub refresh_token: String,
    pub email: String,
    pub last_refresh: String,
    pub expire: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    account: ClaudeAccount,
}

#[derive(Debug, Deserialize)]
struct ClaudeRefreshResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
}

#[derive(Debug, Deserialize)]
struct ClaudeAccount {
    email_address: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsageResponse {
    five_hour: ClaudeUsageWindow,
    seven_day: ClaudeUsageWindow,
    seven_day_sonnet: Option<ClaudeUsageWindow>,
    seven_day_opus: Option<ClaudeUsageWindow>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsageWindow {
    utilization: f64,
    resets_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AntigravityTokenStorage {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub timestamp: i64,
    pub expire: String,
    pub email: String,
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiTokenStorage {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub timestamp: i64,
    pub expire: String,
    pub email: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
}

#[derive(Debug, Deserialize)]
struct GoogleIdTokenClaims {
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FetchAvailableModelsResponse {
    models: HashMap<String, FetchAvailableModelInfo>,
}

#[derive(Debug, Deserialize)]
struct FetchAvailableModelInfo {
    #[serde(rename = "quotaInfo")]
    quota_info: Option<QuotaInfo>,
}

#[derive(Debug, Deserialize)]
struct QuotaInfo {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: f64,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoadCodeAssistResponse {
    #[serde(rename = "cloudaicompanionProject")]
    cloudaicompanion_project: Option<ProjectRef>,
    #[serde(rename = "allowedTiers")]
    allowed_tiers: Option<Vec<TierInfo>>,
}

#[derive(Debug, Deserialize)]
struct TierInfo {
    id: String,
    #[serde(rename = "isDefault", default)]
    is_default: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ProjectRef {
    String(String),
    Object { id: String },
}

#[derive(Debug, Deserialize)]
struct OnboardResponse {
    done: bool,
    response: Option<OnboardResponseData>,
}

#[derive(Debug, Deserialize)]
struct OnboardResponseData {
    #[serde(rename = "cloudaicompanionProject")]
    cloudaicompanion_project: Option<ProjectRef>,
}

#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    email: Option<String>,
    #[serde(rename = "https://api.openai.com/auth")]
    openai_auth: Option<OpenAIAuth>,
}

#[derive(Debug, Deserialize)]
struct OpenAIAuth {
    organizations: Option<Vec<OpenAIOrganization>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIOrganization {
    id: Option<String>,
    uuid: Option<String>,
}

#[derive(Clone)]
struct AuthServerState {
    expected_state: String,
    sender: Arc<Mutex<Option<oneshot::Sender<AuthCallback>>>>,
}

#[derive(Debug)]
struct PendingAuth {
    provider_id: String,
    provider_type: AgentProviderType,
    state: String,
    code_verifier: String,
    receiver: Option<oneshot::Receiver<AuthCallback>>,
    shutdown: Option<oneshot::Sender<()>>,
}

#[derive(Debug)]
struct AuthCallback {
    code: String,
    state: String,
}

#[derive(Deserialize)]
struct AuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

pub struct AgentAuthService {
    store: Arc<ConfigStore>,
    pending: Arc<Mutex<HashMap<String, PendingAuth>>>,
}

impl AgentAuthService {
    pub fn new(store: Arc<ConfigStore>) -> Self {
        Self {
            store,
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_auth(&self, provider_id: &str) -> Result<AgentAuthStart, AgentAuthError> {
        info!("Starting agent auth flow for provider {}", provider_id);
        let provider = self.get_provider(provider_id).await?;
        let agent_type = match provider.provider_type {
            ProviderType::Agent(agent_type) => agent_type,
            _ => return Err(AgentAuthError::NotAgentProvider(provider_id.to_string())),
        };

        let mut pending = self.pending.lock().await;
        if !pending.is_empty() {
            warn!("Auth flow already in progress");
            return Err(AgentAuthError::FlowInProgress);
        }

        let flow_id = Uuid::new_v4().to_string();
        let state = random_state();
        let (auth_url, callback_path, callback_port, code_verifier) = match agent_type {
            AgentProviderType::Codex => {
                let (code_verifier, code_challenge) = generate_pkce_codes();
                let auth_url = build_codex_auth_url(&state, &code_challenge)?;
                (
                    auth_url,
                    CODEX_CALLBACK_PATH,
                    CODEX_CALLBACK_PORT,
                    code_verifier,
                )
            }
            AgentProviderType::ClaudeCode => {
                let (code_verifier, code_challenge) = generate_pkce_codes();
                let auth_url = build_claude_auth_url(&state, &code_challenge)?;
                (
                    auth_url,
                    CLAUDE_CALLBACK_PATH,
                    CLAUDE_CALLBACK_PORT,
                    code_verifier,
                )
            }
            AgentProviderType::Antigravity => {
                let auth_url = build_google_auth_url(
                    ANTIGRAVITY_CLIENT_ID,
                    ANTIGRAVITY_REDIRECT_URI,
                    ANTIGRAVITY_SCOPES,
                    &state,
                )?;
                (
                    auth_url,
                    ANTIGRAVITY_CALLBACK_PATH,
                    ANTIGRAVITY_CALLBACK_PORT,
                    String::new(),
                )
            }
            AgentProviderType::GeminiCli => {
                let auth_url = build_google_auth_url(
                    GEMINI_CLIENT_ID,
                    GEMINI_REDIRECT_URI,
                    GEMINI_SCOPES,
                    &state,
                )?;
                (
                    auth_url,
                    GEMINI_CALLBACK_PATH,
                    GEMINI_CALLBACK_PORT,
                    String::new(),
                )
            }
        };

        let (code_tx, code_rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let server_state = AuthServerState {
            expected_state: state.clone(),
            sender: Arc::new(Mutex::new(Some(code_tx))),
        };

        let app = Router::new()
            .route(callback_path, get(auth_callback))
            .with_state(server_state);

        let listener = TcpListener::bind(("127.0.0.1", callback_port)).await?;
        info!(
            "Auth callback server listening on 127.0.0.1:{}{}",
            callback_port, callback_path
        );
        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        debug!("Auth URL generated for flow {}", flow_id);

        pending.insert(
            flow_id.clone(),
            PendingAuth {
                provider_id: provider_id.to_string(),
                provider_type: agent_type,
                state,
                code_verifier,
                receiver: Some(code_rx),
                shutdown: Some(shutdown_tx),
            },
        );

        Ok(AgentAuthStart { flow_id, auth_url })
    }

    pub async fn complete_auth(&self, flow_id: &str) -> Result<Provider, AgentAuthError> {
        info!("Completing agent auth flow {}", flow_id);
        let pending = {
            let mut pending_map = self.pending.lock().await;
            pending_map
                .remove(flow_id)
                .ok_or_else(|| AgentAuthError::FlowNotFound(flow_id.to_string()))?
        };

        let mut receiver = pending
            .receiver
            .ok_or_else(|| AgentAuthError::FlowNotFound(flow_id.to_string()))?;
        let mut shutdown = pending.shutdown;

        let callback = match tokio::time::timeout(std::time::Duration::from_secs(300), &mut receiver)
            .await
        {
            Ok(Ok(callback)) => callback,
            Ok(Err(_)) => {
                if let Some(shutdown) = shutdown.take() {
                    let _ = shutdown.send(());
                }
                return Err(AgentAuthError::InvalidCallback(
                    "Callback channel closed".to_string(),
                ));
            }
            Err(_) => {
                if let Some(shutdown) = shutdown.take() {
                    let _ = shutdown.send(());
                }
                return Err(AgentAuthError::Timeout);
            }
        };

        if callback.state != pending.state {
            if let Some(shutdown) = shutdown.take() {
                let _ = shutdown.send(());
            }
            return Err(AgentAuthError::InvalidCallback(
                "State mismatch".to_string(),
            ));
        }

        if let Some(shutdown) = shutdown.take() {
            let _ = shutdown.send(());
        }

        debug!(
            "Auth callback received for flow {} (code length {})",
            flow_id,
            callback.code.len()
        );
        match pending.provider_type {
            AgentProviderType::Codex => {
                let token = self
                    .exchange_codex_code(&callback.code, &pending.code_verifier)
                    .await?;

                let id_token = token
                    .id_token
                    .clone()
                    .ok_or_else(|| AgentAuthError::Parse("Missing id_token".to_string()))?;
                let refresh_token = token
                    .refresh_token
                    .clone()
                    .ok_or_else(|| AgentAuthError::Parse("Missing refresh_token".to_string()))?;

                let (account_id, email) = parse_codex_id_token(&id_token)?;

                let now = Utc::now();
                let expire_at = now + ChronoDuration::seconds(token.expires_in);

                let storage = CodexTokenStorage {
                    id_token,
                    access_token: token.access_token,
                    refresh_token,
                    account_id,
                    email: email.clone(),
                    last_refresh: now.to_rfc3339(),
                    expire: expire_at.to_rfc3339(),
                };

                let auth_path = auth_path_for_email(&pending.provider_type, &email)?;
                info!("Saving auth token to {}", auth_path.display());
                save_auth_file(&auth_path, &storage).await?;
                self.update_provider_auth_path(&pending.provider_id, &auth_path, &email)
                    .await?;
            }
            AgentProviderType::ClaudeCode => {
                let token = self
                    .exchange_claude_code(&callback.code, &pending.code_verifier, &pending.state)
                    .await?;
                let email = token.account.email_address;
                let now = Utc::now();
                let expire_at = now + ChronoDuration::seconds(token.expires_in);

                let storage = ClaudeTokenStorage {
                    access_token: token.access_token,
                    refresh_token: token.refresh_token,
                    email: email.clone(),
                    last_refresh: now.to_rfc3339(),
                    expire: expire_at.to_rfc3339(),
                };

                let auth_path = auth_path_for_email(&pending.provider_type, &email)?;
                info!("Saving auth token to {}", auth_path.display());
                save_auth_file(&auth_path, &storage).await?;
                self.update_provider_auth_path(&pending.provider_id, &auth_path, &email)
                    .await?;
            }
            AgentProviderType::Antigravity => {
                let token = self
                    .exchange_google_code(
                        &callback.code,
                        ANTIGRAVITY_CLIENT_ID,
                        ANTIGRAVITY_CLIENT_SECRET,
                        ANTIGRAVITY_REDIRECT_URI,
                    )
                    .await?;
                let access_token = token.access_token;
                let refresh_token = token.refresh_token.ok_or_else(|| {
                    AgentAuthError::Parse("Missing refresh_token".to_string())
                })?;
                let email = match token.id_token.as_deref() {
                    Some(id_token) => match parse_google_id_token(id_token) {
                        Ok(email) => email,
                        Err(err) => {
                            warn!("Failed to parse Google id_token: {}", err);
                            self.fetch_google_email(&access_token).await?
                        }
                    },
                    None => self.fetch_google_email(&access_token).await?,
                };
                let project_id = self.resolve_antigravity_project(&access_token).await?;

                let now = Utc::now();
                let expire_at = now + ChronoDuration::seconds(token.expires_in);
                let storage = AntigravityTokenStorage {
                    access_token,
                    refresh_token,
                    expires_in: token.expires_in,
                    timestamp: now.timestamp_millis(),
                    expire: expire_at.to_rfc3339(),
                    email: email.clone(),
                    project_id,
                };

                let auth_path = auth_path_for_email(&pending.provider_type, &email)?;
                info!("Saving auth token to {}", auth_path.display());
                save_auth_file(&auth_path, &storage).await?;
                self.update_provider_auth_path(&pending.provider_id, &auth_path, &email)
                    .await?;
            }
            AgentProviderType::GeminiCli => {
                let token = self
                    .exchange_google_code(
                        &callback.code,
                        GEMINI_CLIENT_ID,
                        GEMINI_CLIENT_SECRET,
                        GEMINI_REDIRECT_URI,
                    )
                    .await?;
                let access_token = token.access_token;
                let refresh_token = token.refresh_token.ok_or_else(|| {
                    AgentAuthError::Parse("Missing refresh_token".to_string())
                })?;
                let email = match token.id_token.as_deref() {
                    Some(id_token) => match parse_google_id_token(id_token) {
                        Ok(email) => email,
                        Err(err) => {
                            warn!("Failed to parse Google id_token: {}", err);
                            self.fetch_google_email(&access_token).await?
                        }
                    },
                    None => self.fetch_google_email(&access_token).await?,
                };

                let now = Utc::now();
                let expire_at = now + ChronoDuration::seconds(token.expires_in);
                let storage = GeminiTokenStorage {
                    access_token,
                    refresh_token,
                    expires_in: token.expires_in,
                    timestamp: now.timestamp_millis(),
                    expire: expire_at.to_rfc3339(),
                    email: email.clone(),
                    project_id: None,
                };

                let auth_path = auth_path_for_email(&pending.provider_type, &email)?;
                info!("Saving auth token to {}", auth_path.display());
                save_auth_file(&auth_path, &storage).await?;
                self.update_provider_auth_path(&pending.provider_id, &auth_path, &email)
                    .await?;
            }
        }

        self.get_provider(&pending.provider_id).await
    }

    pub async fn get_quota(&self, provider_id: &str) -> Result<AgentQuota, AgentAuthError> {
        let provider = self.get_provider(provider_id).await?;
        let agent_type = match &provider.provider_type {
            ProviderType::Agent(agent_type) => agent_type,
            _ => return Err(AgentAuthError::NotAgentProvider(provider_id.to_string())),
        };

        match *agent_type {
            AgentProviderType::Codex => self.get_codex_quota_for_provider(&provider).await,
            AgentProviderType::ClaudeCode => self.get_claude_quota_for_provider(&provider).await,
            AgentProviderType::Antigravity => {
                self.get_antigravity_quota_for_provider(&provider).await
            }
            AgentProviderType::GeminiCli => self.get_gemini_quota_for_provider(&provider).await,
        }
    }

    async fn get_codex_quota_for_provider(
        &self,
        provider: &Provider,
    ) -> Result<AgentQuota, AgentAuthError> {
        let (auth_path, mut auth): (PathBuf, CodexTokenStorage) = self
            .load_and_normalize_auth(provider, AgentProviderType::Codex)
            .await?;

        if should_refresh_codex(&auth) {
            auth = self.refresh_codex_token(&auth).await?;
            save_auth_file(&auth_path, &auth).await?;
        }

        match self.fetch_codex_quota(&auth).await {
            Ok(quota) => Ok(quota),
            Err(AgentAuthError::Unauthorized) => {
                auth = self.refresh_codex_token(&auth).await?;
                save_auth_file(&auth_path, &auth).await?;
                self.fetch_codex_quota(&auth).await
            }
            Err(err) => Err(err),
        }
    }

    async fn get_claude_quota_for_provider(
        &self,
        provider: &Provider,
    ) -> Result<AgentQuota, AgentAuthError> {
        let (auth_path, mut auth): (PathBuf, ClaudeTokenStorage) = self
            .load_and_normalize_auth(provider, AgentProviderType::ClaudeCode)
            .await?;

        if should_refresh_claude(&auth) {
            auth = self.refresh_claude_token(&auth).await?;
            save_auth_file(&auth_path, &auth).await?;
        }

        match self.fetch_claude_quota(&auth).await {
            Ok(quota) => Ok(quota),
            Err(AgentAuthError::Unauthorized) => {
                auth = self.refresh_claude_token(&auth).await?;
                save_auth_file(&auth_path, &auth).await?;
                self.fetch_claude_quota(&auth).await
            }
            Err(err) => Err(err),
        }
    }

    async fn get_antigravity_quota_for_provider(
        &self,
        provider: &Provider,
    ) -> Result<AgentQuota, AgentAuthError> {
        let (auth_path, mut auth): (PathBuf, AntigravityTokenStorage) = self
            .load_and_normalize_auth(provider, AgentProviderType::Antigravity)
            .await?;

        if should_refresh_google(&auth.timestamp, auth.expires_in) {
            auth = self.refresh_antigravity_token(&auth).await?;
            save_auth_file(&auth_path, &auth).await?;
        }

        match self.fetch_antigravity_quota(&auth).await {
            Ok(quota) => Ok(quota),
            Err(AgentAuthError::Unauthorized) => {
                auth = self.refresh_antigravity_token(&auth).await?;
                save_auth_file(&auth_path, &auth).await?;
                self.fetch_antigravity_quota(&auth).await
            }
            Err(err) => Err(err),
        }
    }

    async fn get_gemini_quota_for_provider(
        &self,
        provider: &Provider,
    ) -> Result<AgentQuota, AgentAuthError> {
        let (auth_path, mut auth): (PathBuf, GeminiTokenStorage) = self
            .load_and_normalize_auth(provider, AgentProviderType::GeminiCli)
            .await?;

        if should_refresh_google(&auth.timestamp, auth.expires_in) {
            auth = self.refresh_gemini_token(&auth).await?;
            save_auth_file(&auth_path, &auth).await?;
        }

        self.fetch_gemini_quota(&auth).await
    }

    async fn fetch_codex_quota(&self, auth: &CodexTokenStorage) -> Result<AgentQuota, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .get(CODEX_USAGE_URL)
            .bearer_auth(&auth.access_token)
            .header("ChatGPT-Account-Id", &auth.account_id)
            .send()
            .await?;

        match response.status() {
            ReqwestStatusCode::UNAUTHORIZED => return Err(AgentAuthError::Unauthorized),
            status if !status.is_success() => {
                let body = response.text().await.unwrap_or_default();
                warn!("Usage request failed: status {} body {}", status, body);
                return Err(AgentAuthError::Parse(format!(
                    "Usage request failed ({}): {}",
                    status, body
                )))
            }
            _ => {}
        }

        let data: CodexUsageResponse = response.json().await?;

        Ok(AgentQuota {
            plan_type: data.plan_type,
            limit_reached: Some(data.rate_limit.limit_reached),
            session_used_percent: data.rate_limit.primary_window.used_percent,
            session_reset_at: Some(data.rate_limit.primary_window.reset_at),
            week_used_percent: data.rate_limit.secondary_window.used_percent,
            week_reset_at: Some(data.rate_limit.secondary_window.reset_at),
            entries: None,
            note: None,
        })
    }

    async fn exchange_codex_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<CodexTokenResponse, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .post(OPENAI_TOKEN_URL)
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", OPENAI_CLIENT_ID),
                ("code", code),
                ("redirect_uri", CODEX_REDIRECT_URI),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Token exchange failed: status {} body {}", status, body);
            return Err(AgentAuthError::Parse(format!(
                "Token exchange failed ({}): {}",
                status, body
            )));
        }

        Ok(response.json().await?)
    }

    async fn refresh_codex_token(
        &self,
        auth: &CodexTokenStorage,
    ) -> Result<CodexTokenStorage, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .post(OPENAI_TOKEN_URL)
            .form(&[
                ("client_id", OPENAI_CLIENT_ID),
                ("grant_type", "refresh_token"),
                ("refresh_token", auth.refresh_token.as_str()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Token refresh failed: status {} body {}", status, body);
            return Err(AgentAuthError::Parse(format!(
                "Token refresh failed ({}): {}",
                status, body
            )));
        }

        let token: CodexTokenResponse = response.json().await?;
        let now = Utc::now();
        let expire_at = now + ChronoDuration::seconds(token.expires_in);
        let id_token = token.id_token.unwrap_or_else(|| auth.id_token.clone());
        let refresh_token = token
            .refresh_token
            .unwrap_or_else(|| auth.refresh_token.clone());

        Ok(CodexTokenStorage {
            id_token,
            access_token: token.access_token,
            refresh_token,
            account_id: auth.account_id.clone(),
            email: auth.email.clone(),
            last_refresh: now.to_rfc3339(),
            expire: expire_at.to_rfc3339(),
        })
    }

    async fn fetch_claude_quota(
        &self,
        auth: &ClaudeTokenStorage,
    ) -> Result<AgentQuota, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .get(CLAUDE_USAGE_URL)
            .bearer_auth(&auth.access_token)
            .header("anthropic-beta", "oauth-2025-04-20")
            .header("Accept", "application/json")
            .send()
            .await?;

        match response.status() {
            ReqwestStatusCode::UNAUTHORIZED => return Err(AgentAuthError::Unauthorized),
            status if !status.is_success() => {
                let body = response.text().await.unwrap_or_default();
                warn!("Claude usage request failed: status {} body {}", status, body);
                return Err(AgentAuthError::Parse(format!(
                    "Claude usage request failed ({}): {}",
                    status, body
                )));
            }
            _ => {}
        }

        let data: ClaudeUsageResponse = response.json().await?;
        let mut entries = Vec::new();

        let five_hour_reset = parse_rfc3339_to_epoch(&data.five_hour.resets_at);
        entries.push(AgentQuotaEntry {
            label: "5h".to_string(),
            used_percent: data.five_hour.utilization,
            reset_at: five_hour_reset,
        });

        let seven_day_reset = parse_rfc3339_to_epoch(&data.seven_day.resets_at);
        entries.push(AgentQuotaEntry {
            label: "7d".to_string(),
            used_percent: data.seven_day.utilization,
            reset_at: seven_day_reset,
        });

        if let Some(window) = data.seven_day_sonnet.as_ref() {
            entries.push(AgentQuotaEntry {
                label: "7d sonnet".to_string(),
                used_percent: window.utilization,
                reset_at: parse_rfc3339_to_epoch(&window.resets_at),
            });
        }
        if let Some(window) = data.seven_day_opus.as_ref() {
            entries.push(AgentQuotaEntry {
                label: "7d opus".to_string(),
                used_percent: window.utilization,
                reset_at: parse_rfc3339_to_epoch(&window.resets_at),
            });
        }

        Ok(AgentQuota {
            plan_type: Some("Claude Code".to_string()),
            limit_reached: None,
            session_used_percent: data.five_hour.utilization,
            session_reset_at: five_hour_reset,
            week_used_percent: data.seven_day.utilization,
            week_reset_at: seven_day_reset,
            entries: Some(entries),
            note: None,
        })
    }

    async fn exchange_claude_code(
        &self,
        code: &str,
        code_verifier: &str,
        state: &str,
    ) -> Result<ClaudeTokenResponse, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .post(ANTHROPIC_TOKEN_URL)
            .json(&serde_json::json!({
                "code": code,
                "state": state,
                "grant_type": "authorization_code",
                "client_id": ANTHROPIC_CLIENT_ID,
                "redirect_uri": CLAUDE_REDIRECT_URI,
                "code_verifier": code_verifier,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Claude token exchange failed: status {} body {}", status, body);
            return Err(AgentAuthError::Parse(format!(
                "Claude token exchange failed ({}): {}",
                status, body
            )));
        }

        Ok(response.json().await?)
    }

    async fn refresh_claude_token(
        &self,
        auth: &ClaudeTokenStorage,
    ) -> Result<ClaudeTokenStorage, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .post(ANTHROPIC_TOKEN_URL)
            .json(&serde_json::json!({
                "client_id": ANTHROPIC_CLIENT_ID,
                "grant_type": "refresh_token",
                "refresh_token": auth.refresh_token.clone(),
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Claude token refresh failed: status {} body {}", status, body);
            return Err(AgentAuthError::Parse(format!(
                "Claude token refresh failed ({}): {}",
                status, body
            )));
        }

        let token: ClaudeRefreshResponse = response.json().await?;
        let now = Utc::now();
        let expire_at = now + ChronoDuration::seconds(token.expires_in);

        Ok(ClaudeTokenStorage {
            access_token: token.access_token,
            refresh_token: token
                .refresh_token
                .unwrap_or_else(|| auth.refresh_token.clone()),
            email: auth.email.clone(),
            last_refresh: now.to_rfc3339(),
            expire: expire_at.to_rfc3339(),
        })
    }

    async fn exchange_google_code(
        &self,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> Result<GoogleTokenResponse, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("code", code),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Google token exchange failed: status {} body {}", status, body);
            return Err(AgentAuthError::Parse(format!(
                "Google token exchange failed ({}): {}",
                status, body
            )));
        }

        Ok(response.json().await?)
    }

    async fn refresh_antigravity_token(
        &self,
        auth: &AntigravityTokenStorage,
    ) -> Result<AntigravityTokenStorage, AgentAuthError> {
        let token = self
            .refresh_google_token(
                &auth.refresh_token,
                ANTIGRAVITY_CLIENT_ID,
                ANTIGRAVITY_CLIENT_SECRET,
            )
            .await?;
        let now = Utc::now();
        let expire_at = now + ChronoDuration::seconds(token.expires_in);

        Ok(AntigravityTokenStorage {
            access_token: token.access_token,
            refresh_token: token
                .refresh_token
                .unwrap_or_else(|| auth.refresh_token.clone()),
            expires_in: token.expires_in,
            timestamp: now.timestamp_millis(),
            expire: expire_at.to_rfc3339(),
            email: auth.email.clone(),
            project_id: auth.project_id.clone(),
        })
    }

    async fn refresh_gemini_token(
        &self,
        auth: &GeminiTokenStorage,
    ) -> Result<GeminiTokenStorage, AgentAuthError> {
        let token = self
            .refresh_google_token(&auth.refresh_token, GEMINI_CLIENT_ID, GEMINI_CLIENT_SECRET)
            .await?;
        let now = Utc::now();
        let expire_at = now + ChronoDuration::seconds(token.expires_in);

        Ok(GeminiTokenStorage {
            access_token: token.access_token,
            refresh_token: token
                .refresh_token
                .unwrap_or_else(|| auth.refresh_token.clone()),
            expires_in: token.expires_in,
            timestamp: now.timestamp_millis(),
            expire: expire_at.to_rfc3339(),
            email: auth.email.clone(),
            project_id: auth.project_id.clone(),
        })
    }

    async fn refresh_google_token(
        &self,
        refresh_token: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<GoogleTokenResponse, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Google token refresh failed: status {} body {}", status, body);
            return Err(AgentAuthError::Parse(format!(
                "Google token refresh failed ({}): {}",
                status, body
            )));
        }

        Ok(response.json().await?)
    }

    async fn fetch_antigravity_quota(
        &self,
        auth: &AntigravityTokenStorage,
    ) -> Result<AgentQuota, AgentAuthError> {
        let mut body = json!({});
        if !auth.project_id.is_empty() {
            body["project"] = json!(auth.project_id.clone());
        }

        let response = self
            .http_client()
            .await?
            .post(ANTIGRAVITY_FETCH_MODELS_URL)
            .bearer_auth(&auth.access_token)
            .header("User-Agent", "antigravity/1.11.3 Darwin/arm64")
            .json(&body)
            .send()
            .await?;

        match response.status() {
            ReqwestStatusCode::UNAUTHORIZED => return Err(AgentAuthError::Unauthorized),
            status if !status.is_success() => {
                let body = response.text().await.unwrap_or_default();
                warn!("Antigravity quota request failed: status {} body {}", status, body);
                return Err(AgentAuthError::Parse(format!(
                    "Antigravity quota request failed ({}): {}",
                    status, body
                )));
            }
            _ => {}
        }

        let data: FetchAvailableModelsResponse = response.json().await?;
        let mut entries: Vec<AgentQuotaEntry> = data
            .models
            .into_iter()
            .filter_map(|(name, model)| {
                model.quota_info.map(|quota| AgentQuotaEntry {
                    label: name,
                    used_percent: (1.0 - quota.remaining_fraction) * 100.0,
                    reset_at: quota
                        .reset_time
                        .as_deref()
                        .and_then(parse_rfc3339_to_epoch),
                })
            })
            .collect();

        entries.sort_by(|a, b| a.label.cmp(&b.label));

        let session = entries.first();
        let week = entries.get(1).or(session);
        let note = if entries.is_empty() {
            Some("No quota data returned for this project.".to_string())
        } else {
            None
        };

        Ok(AgentQuota {
            plan_type: Some("Antigravity".to_string()),
            limit_reached: None,
            session_used_percent: session.map(|e| e.used_percent).unwrap_or(0.0),
            session_reset_at: session.and_then(|e| e.reset_at),
            week_used_percent: week.map(|e| e.used_percent).unwrap_or(0.0),
            week_reset_at: week.and_then(|e| e.reset_at),
            entries: Some(entries),
            note,
        })
    }

    async fn fetch_gemini_quota(
        &self,
        _auth: &GeminiTokenStorage,
    ) -> Result<AgentQuota, AgentAuthError> {
        Ok(AgentQuota {
            plan_type: Some("Google Account".to_string()),
            limit_reached: None,
            session_used_percent: 0.0,
            session_reset_at: None,
            week_used_percent: 0.0,
            week_reset_at: None,
            entries: None,
            note: Some("Gemini CLI does not expose a quota API yet.".to_string()),
        })
    }

    async fn fetch_google_email(&self, access_token: &str) -> Result<String, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|err| {
                warn!("Google userinfo request failed: {}", err);
                AgentAuthError::Http(err)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Google userinfo failed: status {} body {}", status, body);
            return Err(AgentAuthError::Parse(format!(
                "Google userinfo failed ({}): {}",
                status, body
            )));
        }

        let data: GoogleUserInfo = response.json().await?;
        Ok(data.email)
    }

    async fn resolve_antigravity_project(
        &self,
        access_token: &str,
    ) -> Result<String, AgentAuthError> {
        let response = self.load_code_assist(access_token).await?;
        let LoadCodeAssistResponse {
            cloudaicompanion_project,
            allowed_tiers,
        } = response;
        if let Some(project) = cloudaicompanion_project.and_then(project_ref_to_id) {
            return Ok(project);
        }

        let tiers = allowed_tiers.unwrap_or_default();
        let tier_id = tiers
            .iter()
            .find(|tier| tier.is_default)
            .map(|tier| tier.id.clone())
            .or_else(|| tiers.first().map(|tier| tier.id.clone()))
            .ok_or_else(|| AgentAuthError::Parse("No available tier".to_string()))?;

        self.onboard_user(access_token, &tier_id).await
    }

    async fn load_code_assist(
        &self,
        access_token: &str,
    ) -> Result<LoadCodeAssistResponse, AgentAuthError> {
        let response = self
            .http_client()
            .await?
            .post(ANTIGRAVITY_LOAD_CODE_ASSIST_URL)
            .bearer_auth(access_token)
            .json(&json!({
                "metadata": {
                    "ideType": "ANTIGRAVITY",
                    "platform": "PLATFORM_UNSPECIFIED",
                    "pluginType": "GEMINI"
                }
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("loadCodeAssist failed: status {} body {}", status, body);
            return Err(AgentAuthError::Parse(format!(
                "loadCodeAssist failed ({}): {}",
                status, body
            )));
        }

        Ok(response.json().await?)
    }

    async fn onboard_user(
        &self,
        access_token: &str,
        tier_id: &str,
    ) -> Result<String, AgentAuthError> {
        for attempt in 1..=5 {
            let response = self
                .http_client()
                .await?
                .post(ANTIGRAVITY_ONBOARD_USER_URL)
                .bearer_auth(access_token)
                .json(&json!({
                    "tierId": tier_id,
                    "metadata": {
                        "ideType": "ANTIGRAVITY",
                        "platform": "PLATFORM_UNSPECIFIED",
                        "pluginType": "GEMINI"
                    }
                }))
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                warn!("onboardUser failed: status {} body {}", status, body);
                return Err(AgentAuthError::Parse(format!(
                    "onboardUser failed ({}): {}",
                    status, body
                )));
            }

            let data: OnboardResponse = response.json().await?;
            if data.done {
                if let Some(project) = data
                    .response
                    .and_then(|resp| resp.cloudaicompanion_project)
                    .and_then(project_ref_to_id)
                {
                    return Ok(project);
                }
                return Err(AgentAuthError::Parse(
                    "Onboarding succeeded without project id".to_string(),
                ));
            }

            if attempt < 5 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }

        Err(AgentAuthError::Parse("Onboarding timeout".to_string()))
    }

    async fn get_provider(&self, id: &str) -> Result<Provider, AgentAuthError> {
        let config = self.store.get_config().await;
        config
            .providers
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| AgentAuthError::ProviderNotFound(id.to_string()))
    }

    async fn update_provider_auth_path(
        &self,
        provider_id: &str,
        auth_path: &PathBuf,
        email: &str,
    ) -> Result<(), AgentAuthError> {
        let id = provider_id.to_string();
        let auth_path_string = auth_path.to_string_lossy().to_string();
        let email_string = email.to_string();
        self.store
            .update(|config| {
                if let Some(provider) = config.providers.iter_mut().find(|p| p.id == id) {
                    provider.auth_path = Some(auth_path_string.clone());
                    provider.auth_email = Some(email_string.clone());
                    provider.status = ProviderStatus::Connected;
                    provider.updated_at = Utc::now();
                }
            })
            .await?;
        Ok(())
    }

    async fn load_and_normalize_auth<T>(
        &self,
        provider: &Provider,
        agent_type: AgentProviderType,
    ) -> Result<(PathBuf, T), AgentAuthError>
    where
        T: DeserializeOwned + AuthEmail,
    {
        let auth_path = provider
            .auth_path
            .clone()
            .ok_or_else(|| AgentAuthError::Parse("Auth path not set. Please login again.".to_string()))?;
        let mut auth_path = PathBuf::from(auth_path);
        if !auth_path.exists() {
            return Err(AgentAuthError::Parse(
                "Auth file not found. Please login again.".to_string(),
            ));
        }
        debug!("Loading auth token from {}", auth_path.display());
        let auth: T = load_auth_file(&auth_path).await?;

        let desired_path = auth_path_for_email(&agent_type, auth.email())?;
        if desired_path != auth_path {
            let mut final_path = desired_path.clone();
            if !final_path.exists() {
                match tokio::fs::rename(&auth_path, &final_path).await {
                    Ok(()) => {
                        info!("Renamed auth file to {}", final_path.display());
                    }
                    Err(err) => {
                        warn!(
                            "Failed to rename auth file from {} to {}: {}",
                            auth_path.display(),
                            final_path.display(),
                            err
                        );
                        final_path = auth_path.clone();
                    }
                }
            }
            if final_path != auth_path && final_path.exists() {
                self.update_provider_auth_path(&provider.id, &final_path, auth.email())
                    .await?;
            }
            auth_path = final_path;
        } else if provider
            .auth_email
            .as_deref()
            .map(|email| email != auth.email())
            .unwrap_or(true)
        {
            self.update_provider_auth_path(&provider.id, &auth_path, auth.email())
                .await?;
        }

        Ok((auth_path, auth))
    }

    async fn http_client(&self) -> Result<reqwest::Client, AgentAuthError> {
        let config = self.store.get_config().await;
        let mut builder = reqwest::Client::builder();

        if config.app.enable_proxy {
            let host = config.app.proxy_host.clone().unwrap_or_default();
            let port = config.app.proxy_port.unwrap_or_default();
            if !host.is_empty() && port > 0 {
                let proxy_url = format!("http://{}:{}", host, port);
                let mut proxy = Proxy::all(&proxy_url)
                    .map_err(|err| AgentAuthError::Parse(err.to_string()))?;
                if !config.app.no_proxy.is_empty() {
                    let no_proxy = NoProxy::from_string(&config.app.no_proxy.join(","));
                    proxy = proxy.no_proxy(no_proxy);
                }
                builder = builder.proxy(proxy);
                debug!("Using proxy {} for agent auth requests", proxy_url);
            } else {
                warn!("Proxy enabled but host/port not configured");
            }
        }

        builder
            .build()
            .map_err(|err| AgentAuthError::Parse(err.to_string()))
    }
}

fn generate_pkce_codes() -> (String, String) {
    let verifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    let challenge = URL_SAFE_NO_PAD.encode(hash);

    (verifier, challenge)
}

fn random_state() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

fn build_codex_auth_url(state: &str, code_challenge: &str) -> Result<String, AgentAuthError> {
    let mut url = reqwest::Url::parse(OPENAI_AUTH_URL)
        .map_err(|err| AgentAuthError::Parse(err.to_string()))?;

    let scope = CODEX_SCOPES.join(" ");

    url.query_pairs_mut()
        .append_pair("client_id", OPENAI_CLIENT_ID)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", CODEX_REDIRECT_URI)
        .append_pair("scope", &scope)
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("id_token_add_organizations", "true")
        .append_pair("codex_cli_simplified_flow", "true")
        .append_pair("originator", ORIGINATOR);

    Ok(url.to_string())
}

fn build_claude_auth_url(state: &str, code_challenge: &str) -> Result<String, AgentAuthError> {
    let mut url = reqwest::Url::parse(ANTHROPIC_AUTH_URL)
        .map_err(|err| AgentAuthError::Parse(err.to_string()))?;

    let scope = CLAUDE_SCOPES.join(" ");

    url.query_pairs_mut()
        .append_pair("code", "true")
        .append_pair("client_id", ANTHROPIC_CLIENT_ID)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", CLAUDE_REDIRECT_URI)
        .append_pair("scope", &scope)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", state);

    Ok(url.to_string())
}

fn build_google_auth_url(
    client_id: &str,
    redirect_uri: &str,
    scopes: &[&str],
    state: &str,
) -> Result<String, AgentAuthError> {
    let mut url =
        reqwest::Url::parse(GOOGLE_AUTH_URL).map_err(|err| AgentAuthError::Parse(err.to_string()))?;
    let scope = scopes.join(" ");

    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", &scope)
        .append_pair("response_type", "code")
        .append_pair("state", state)
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");

    Ok(url.to_string())
}

fn split_code_and_state(code: &str) -> (String, Option<String>) {
    if let Some((left, right)) = code.split_once('#') {
        let mut state_value = right.trim();
        if let Some(stripped) = state_value.strip_prefix("state=") {
            state_value = stripped;
        }
        let state = if state_value.is_empty() {
            None
        } else {
            Some(state_value.to_string())
        };
        return (left.to_string(), state);
    }
    (code.to_string(), None)
}

fn parse_codex_id_token(id_token: &str) -> Result<(String, String), AgentAuthError> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err(AgentAuthError::Parse("Invalid JWT format".to_string()));
    }

    let payload = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|err| AgentAuthError::Parse(err.to_string()))?;
    let claims: IdTokenClaims =
        serde_json::from_slice(&payload).map_err(|err| AgentAuthError::Parse(err.to_string()))?;

    let email = claims
        .email
        .ok_or_else(|| AgentAuthError::Parse("Missing email in id_token".to_string()))?;

    let account_id = claims
        .openai_auth
        .and_then(|auth| auth.organizations)
        .and_then(|mut orgs| orgs.pop())
        .and_then(|org| org.id.or(org.uuid))
        .ok_or_else(|| AgentAuthError::Parse("Missing account id".to_string()))?;

    Ok((account_id, email))
}

fn parse_google_id_token(id_token: &str) -> Result<String, AgentAuthError> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err(AgentAuthError::Parse("Invalid JWT format".to_string()));
    }

    let payload = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|err| AgentAuthError::Parse(err.to_string()))?;
    let claims: GoogleIdTokenClaims =
        serde_json::from_slice(&payload).map_err(|err| AgentAuthError::Parse(err.to_string()))?;

    let email = claims
        .email
        .ok_or_else(|| AgentAuthError::Parse("Missing email in id_token".to_string()))?;

    Ok(email)
}

fn should_refresh_codex(auth: &CodexTokenStorage) -> bool {
    let expire = DateTime::parse_from_rfc3339(&auth.expire)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    expire - Utc::now() < ChronoDuration::days(5)
}

fn should_refresh_claude(auth: &ClaudeTokenStorage) -> bool {
    let expire = DateTime::parse_from_rfc3339(&auth.expire)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    expire - Utc::now() < ChronoDuration::minutes(5)
}

fn should_refresh_google(timestamp: &i64, expires_in: i64) -> bool {
    let now_ms = Utc::now().timestamp_millis();
    let expiry = *timestamp + (expires_in * 1000);
    let refresh_skew = 3000 * 1000;
    now_ms >= (expiry - refresh_skew)
}

fn parse_rfc3339_to_epoch(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.timestamp())
        .ok()
}

fn project_ref_to_id(project: ProjectRef) -> Option<String> {
    match project {
        ProjectRef::String(value) => Some(value),
        ProjectRef::Object { id } => Some(id),
    }
}

fn auth_path_for_email(
    agent_type: &AgentProviderType,
    email: &str,
) -> Result<PathBuf, AgentAuthError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AgentAuthError::Parse("Could not determine home directory".to_string()))?;
    let sanitized_email: String = email
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '@' || ch == '.' || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let filename = match agent_type {
        AgentProviderType::Codex => format!("codex_{}.json", sanitized_email),
        AgentProviderType::ClaudeCode => format!("claude-code_{}.json", sanitized_email),
        AgentProviderType::GeminiCli => format!("gemini-cli_{}.json", sanitized_email),
        AgentProviderType::Antigravity => format!("antigravity_{}.json", sanitized_email),
    };
    Ok(home.join(".vibemate").join("auth").join(filename))
}

trait AuthEmail {
    fn email(&self) -> &str;
}

impl AuthEmail for CodexTokenStorage {
    fn email(&self) -> &str {
        &self.email
    }
}

impl AuthEmail for ClaudeTokenStorage {
    fn email(&self) -> &str {
        &self.email
    }
}

impl AuthEmail for AntigravityTokenStorage {
    fn email(&self) -> &str {
        &self.email
    }
}

impl AuthEmail for GeminiTokenStorage {
    fn email(&self) -> &str {
        &self.email
    }
}

async fn save_auth_file<T: Serialize>(path: &PathBuf, auth: &T) -> Result<(), AgentAuthError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = serde_json::to_string_pretty(auth)
        .map_err(|err| AgentAuthError::Parse(err.to_string()))?;
    tokio::fs::write(path, content).await?;
    Ok(())
}

async fn load_auth_file<T: DeserializeOwned>(path: &PathBuf) -> Result<T, AgentAuthError> {
    let content = tokio::fs::read_to_string(path).await?;
    serde_json::from_str(&content).map_err(|err| AgentAuthError::Parse(err.to_string()))
}

async fn auth_callback(
    State(state): State<AuthServerState>,
    Query(params): Query<AuthCallbackQuery>,
) -> impl IntoResponse {
    let code = match params.code {
        Some(code) => code,
        None => {
            return (
                AxumStatusCode::BAD_REQUEST,
                "Missing code in callback",
            )
                .into_response()
        }
    };

    let (clean_code, state_from_code) = split_code_and_state(&code);
    let callback_state = params.state.or(state_from_code);
    let callback_state = match callback_state {
        Some(state) => state,
        None => {
            return (
                AxumStatusCode::BAD_REQUEST,
                "Missing state in callback",
            )
                .into_response()
        }
    };

    if callback_state != state.expected_state {
        return (
            AxumStatusCode::BAD_REQUEST,
            "Invalid state in callback",
        )
            .into_response();
    }

    if let Some(sender) = state.sender.lock().await.take() {
        let _ = sender.send(AuthCallback {
            code: clean_code,
            state: callback_state.clone(),
        });
    } else {
        warn!("Auth callback received but sender already used");
    }

    Html(
        r#"Authentication successful. You can close this window and return to Vibe Mate."#,
    )
    .into_response()
}
