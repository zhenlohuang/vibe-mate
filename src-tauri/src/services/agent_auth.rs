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
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;
use tracing::{debug, error, info, warn};

use crate::models::{
    AgentAuthStart, AgentProviderType, AgentQuota, Provider, ProviderStatus, ProviderType,
};
use crate::storage::ConfigStore;

const OPENAI_AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const OPENAI_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const OPENAI_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const CALLBACK_PORT: u16 = 1455;
const ORIGINATOR: &str = "codex_cli_rs";
const CODEX_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

const SCOPES: &[&str] = &["openid", "email", "profile", "offline_access"];

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

        if agent_type != AgentProviderType::Codex {
            return Err(AgentAuthError::UnsupportedAgentProvider(format!(
                "{:?}",
                agent_type
            )));
        }

        let mut pending = self.pending.lock().await;
        if !pending.is_empty() {
            warn!("Auth flow already in progress");
            return Err(AgentAuthError::FlowInProgress);
        }

        let flow_id = Uuid::new_v4().to_string();
        let (code_verifier, code_challenge) = generate_pkce_codes();
        let state = random_state();

        let (code_tx, code_rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let server_state = AuthServerState {
            expected_state: state.clone(),
            sender: Arc::new(Mutex::new(Some(code_tx))),
        };

        let app = Router::new()
            .route("/auth/callback", get(auth_callback))
            .with_state(server_state);

        let listener = TcpListener::bind(("127.0.0.1", CALLBACK_PORT)).await?;
        info!("Auth callback server listening on 127.0.0.1:{}", CALLBACK_PORT);
        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        let auth_url = build_codex_auth_url(&state, &code_challenge)?;
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

        info!(
            "Updating provider auth path for provider {}",
            pending.provider_id
        );
        self.update_provider_auth_path(&pending.provider_id, &auth_path, &email)
            .await?;

        self.get_provider(&pending.provider_id).await
    }

    pub async fn get_quota(&self, provider_id: &str) -> Result<AgentQuota, AgentAuthError> {
        let provider = self.get_provider(provider_id).await?;
        let agent_type = match provider.provider_type {
            ProviderType::Agent(agent_type) => agent_type,
            _ => return Err(AgentAuthError::NotAgentProvider(provider_id.to_string())),
        };

        if agent_type != AgentProviderType::Codex {
            return Err(AgentAuthError::UnsupportedAgentProvider(format!(
                "{:?}",
                agent_type
            )));
        }

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
        let mut auth = load_auth_file(&auth_path).await?;

        let desired_path = auth_path_for_email(&agent_type, &auth.email)?;
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
                self.update_provider_auth_path(&provider.id, &final_path, &auth.email)
                    .await?;
            }
            auth_path = final_path;
        } else if provider
            .auth_email
            .as_deref()
            .map(|email| email != auth.email)
            .unwrap_or(true)
        {
            self.update_provider_auth_path(&provider.id, &auth_path, &auth.email)
                .await?;
        }

        if should_refresh(&auth) {
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
                ("redirect_uri", REDIRECT_URI),
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

    let scope = SCOPES.join(" ");

    url.query_pairs_mut()
        .append_pair("client_id", OPENAI_CLIENT_ID)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("scope", &scope)
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("id_token_add_organizations", "true")
        .append_pair("codex_cli_simplified_flow", "true")
        .append_pair("originator", ORIGINATOR);

    Ok(url.to_string())
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

fn should_refresh(auth: &CodexTokenStorage) -> bool {
    let expire = DateTime::parse_from_rfc3339(&auth.expire)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    expire - Utc::now() < ChronoDuration::days(5)
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

async fn save_auth_file(path: &PathBuf, auth: &CodexTokenStorage) -> Result<(), AgentAuthError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = serde_json::to_string_pretty(auth)
        .map_err(|err| AgentAuthError::Parse(err.to_string()))?;
    tokio::fs::write(path, content).await?;
    Ok(())
}

async fn load_auth_file(path: &PathBuf) -> Result<CodexTokenStorage, AgentAuthError> {
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

    let callback_state = match params.state {
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
            code,
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
