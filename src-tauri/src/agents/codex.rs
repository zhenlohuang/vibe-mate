use crate::agents::{
    auth::{auth_path_for_email, generate_pkce_codes, save_auth_file, AuthEmail, AuthFlowStart},
    auth::{AgentAuthContext, AgentAuthError},
    binary_is_installed, resolve_binary_version, AgentMetadata, CodingAgentDefinition,
};
use crate::models::{AgentProviderType, AgentQuota, AgentType, Provider};

use base64::Engine as _;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reqwest::StatusCode as ReqwestStatusCode;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

const OPENAI_AUTH_URL: &str = "https://auth.openai.com/oauth/authorize";
const OPENAI_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const OPENAI_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const CODEX_REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const CODEX_CALLBACK_PATH: &str = "/auth/callback";
const CODEX_CALLBACK_PORT: u16 = 1455;
const ORIGINATOR: &str = "codex_cli_rs";
const CODEX_USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

const CODEX_SCOPES: &[&str] = &["openid", "email", "profile", "offline_access"];

pub struct CodexAgent;

impl CodexAgent {
    pub const METADATA: AgentMetadata = AgentMetadata {
        agent_type: AgentType::Codex,
        name: "Codex",
        binary: "codex",
        default_config_file: "~/.codex/config.toml",
        default_auth_file: "~/.codex/auth.json",
    };
}

impl CodingAgentDefinition for CodexAgent {
    fn metadata(&self) -> &'static AgentMetadata {
        &Self::METADATA
    }

    fn is_installed(&self) -> bool {
        binary_is_installed(Self::METADATA.binary)
    }

    fn get_version(&self) -> Option<String> {
        resolve_binary_version(Self::METADATA.binary)
    }
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

impl AuthEmail for CodexTokenStorage {
    fn email(&self) -> &str {
        &self.email
    }
}

pub(crate) fn start_auth_flow(state: &str) -> Result<AuthFlowStart, AgentAuthError> {
    let (code_verifier, code_challenge) = generate_pkce_codes();
    let auth_url = build_codex_auth_url(state, &code_challenge)?;
    Ok(AuthFlowStart {
        auth_url,
        callback_path: CODEX_CALLBACK_PATH,
        callback_port: CODEX_CALLBACK_PORT,
        code_verifier,
    })
}

pub(crate) async fn complete_auth(
    ctx: &AgentAuthContext,
    provider_id: &str,
    _state: &str,
    code: &str,
    code_verifier: &str,
) -> Result<(), AgentAuthError> {
    let token = exchange_codex_code(ctx, code, code_verifier).await?;

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

    let auth_path = auth_path_for_email(&AgentProviderType::Codex, &email)?;
    info!("Saving auth token to {}", auth_path.display());
    save_auth_file(&auth_path, &storage).await?;
    ctx.update_provider_auth_path(provider_id, &auth_path, &email)
        .await?;

    Ok(())
}

pub(crate) async fn get_quota(
    ctx: &AgentAuthContext,
    provider: &Provider,
) -> Result<AgentQuota, AgentAuthError> {
    let (auth_path, mut auth): (std::path::PathBuf, CodexTokenStorage) = ctx
        .load_and_normalize_auth(provider, AgentProviderType::Codex)
        .await?;

    if should_refresh_codex(&auth) {
        auth = refresh_codex_token(ctx, &auth).await?;
        save_auth_file(&auth_path, &auth).await?;
    }

    match fetch_codex_quota(ctx, &auth).await {
        Ok(quota) => Ok(quota),
        Err(AgentAuthError::Unauthorized) => {
            auth = refresh_codex_token(ctx, &auth).await?;
            save_auth_file(&auth_path, &auth).await?;
            fetch_codex_quota(ctx, &auth).await
        }
        Err(err) => Err(err),
    }
}

async fn fetch_codex_quota(
    ctx: &AgentAuthContext,
    auth: &CodexTokenStorage,
) -> Result<AgentQuota, AgentAuthError> {
    let response = ctx
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
            )));
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
    ctx: &AgentAuthContext,
    code: &str,
    code_verifier: &str,
) -> Result<CodexTokenResponse, AgentAuthError> {
    let response = ctx
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
    ctx: &AgentAuthContext,
    auth: &CodexTokenStorage,
) -> Result<CodexTokenStorage, AgentAuthError> {
    let response = ctx
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

fn parse_codex_id_token(id_token: &str) -> Result<(String, String), AgentAuthError> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return Err(AgentAuthError::Parse("Invalid JWT format".to_string()));
    }

    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
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

fn should_refresh_codex(auth: &CodexTokenStorage) -> bool {
    let expire = DateTime::parse_from_rfc3339(&auth.expire)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    expire - Utc::now() < ChronoDuration::days(5)
}
