use std::path::PathBuf;
use std::sync::Arc;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{NoProxy, Proxy};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::models::{AgentProviderType, Provider, ProviderStatus};
use crate::storage::ConfigStore;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v1/userinfo?alt=json";

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

#[derive(Debug, Clone)]
pub struct AuthFlowStart {
    pub auth_url: String,
    pub callback_path: &'static str,
    pub callback_port: u16,
    pub code_verifier: String,
}

pub trait AuthEmail {
    fn email(&self) -> &str;
}

#[derive(Clone)]
pub struct AgentAuthContext {
    store: Arc<ConfigStore>,
}

impl AgentAuthContext {
    pub fn new(store: Arc<ConfigStore>) -> Self {
        Self { store }
    }

    pub async fn get_provider(&self, id: &str) -> Result<Provider, AgentAuthError> {
        let config = self.store.get_config().await;
        config
            .providers
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| AgentAuthError::ProviderNotFound(id.to_string()))
    }

    pub async fn update_provider_auth_path(
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

    pub async fn load_and_normalize_auth<T>(
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

    pub async fn http_client(&self) -> Result<reqwest::Client, AgentAuthError> {
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

    pub async fn fetch_google_email(&self, access_token: &str) -> Result<String, AgentAuthError> {
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
}

#[derive(Debug, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
}

#[derive(Debug, Deserialize)]
struct GoogleIdTokenClaims {
    email: Option<String>,
}

pub fn generate_pkce_codes() -> (String, String) {
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

pub fn random_state() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub fn build_google_auth_url(
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

pub async fn exchange_google_code(
    ctx: &AgentAuthContext,
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
) -> Result<GoogleTokenResponse, AgentAuthError> {
    let response = ctx
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

pub async fn refresh_google_token(
    ctx: &AgentAuthContext,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<GoogleTokenResponse, AgentAuthError> {
    let response = ctx
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

pub fn parse_google_id_token(id_token: &str) -> Result<String, AgentAuthError> {
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

pub fn should_refresh_google(timestamp: &i64, expires_in: i64) -> bool {
    let now_ms = Utc::now().timestamp_millis();
    let expiry = *timestamp + (expires_in * 1000);
    let refresh_skew = 3000 * 1000;
    now_ms >= (expiry - refresh_skew)
}

pub fn parse_rfc3339_to_epoch(value: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.timestamp())
        .ok()
}

pub fn auth_path_for_email(
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

pub async fn save_auth_file<T: Serialize>(
    path: &PathBuf,
    auth: &T,
) -> Result<(), AgentAuthError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = serde_json::to_string_pretty(auth)
        .map_err(|err| AgentAuthError::Parse(err.to_string()))?;
    tokio::fs::write(path, content).await?;
    Ok(())
}

pub async fn load_auth_file<T: DeserializeOwned>(path: &PathBuf) -> Result<T, AgentAuthError> {
    let content = tokio::fs::read_to_string(path).await?;
    serde_json::from_str(&content).map_err(|err| AgentAuthError::Parse(err.to_string()))
}
