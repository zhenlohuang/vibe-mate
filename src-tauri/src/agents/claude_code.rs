use crate::agents::{
    auth::{
        auth_path_for_agent_type, generate_pkce_codes, parse_rfc3339_to_epoch, save_auth_file,
    },
    auth::{AgentAuthContext, AgentAuthError, AuthFlowStart},
    AgentMetadata, CodingAgentDefinition,
};
use crate::models::{AgentProviderType, AgentQuota, AgentQuotaEntry, AgentType};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reqwest::StatusCode as ReqwestStatusCode;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

const ANTHROPIC_AUTH_URL: &str = "https://claude.ai/oauth/authorize";
const ANTHROPIC_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const ANTHROPIC_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const CLAUDE_REDIRECT_URI: &str = "http://localhost:54545/callback";
const CLAUDE_CALLBACK_PATH: &str = "/callback";
const CLAUDE_CALLBACK_PORT: u16 = 54545;
const CLAUDE_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

const CLAUDE_SCOPES: &[&str] = &["org:create_api_key", "user:profile", "user:inference"];

pub struct ClaudeCodeAgent;

impl ClaudeCodeAgent {
    pub const METADATA: AgentMetadata = AgentMetadata {
        agent_type: AgentType::ClaudeCode,
        name: "Claude Code",
        binary: "claude",
        default_config_file: "~/.claude/settings.json",
        default_auth_file: "~/.claude/credentials.json",
    };
}

impl CodingAgentDefinition for ClaudeCodeAgent {
    fn metadata(&self) -> &'static AgentMetadata {
        &Self::METADATA
    }
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
    resets_at: Option<String>,
}

pub(crate) fn start_auth_flow(state: &str) -> Result<AuthFlowStart, AgentAuthError> {
    let (code_verifier, code_challenge) = generate_pkce_codes();
    let auth_url = build_claude_auth_url(state, &code_challenge)?;
    Ok(AuthFlowStart {
        auth_url,
        callback_path: CLAUDE_CALLBACK_PATH,
        callback_port: CLAUDE_CALLBACK_PORT,
        code_verifier,
    })
}

pub(crate) async fn complete_auth(
    ctx: &AgentAuthContext,
    agent_type: &AgentProviderType,
    state: &str,
    code: &str,
    code_verifier: &str,
) -> Result<(), AgentAuthError> {
    let token = exchange_claude_code(ctx, code, code_verifier, state).await?;
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

    let auth_path = auth_path_for_agent_type(agent_type)?;
    info!("Saving auth token to {}", auth_path.display());
    save_auth_file(&auth_path, &storage).await?;

    Ok(())
}

pub(crate) async fn get_quota(
    ctx: &AgentAuthContext,
    agent_type: &AgentProviderType,
) -> Result<AgentQuota, AgentAuthError> {
    let (auth_path, mut auth): (std::path::PathBuf, ClaudeTokenStorage) = ctx
        .load_and_normalize_auth(agent_type)
        .await?;

    if should_refresh_claude(&auth) {
        auth = refresh_claude_token(ctx, &auth).await?;
        save_auth_file(&auth_path, &auth).await?;
    }

    match fetch_claude_quota(ctx, &auth).await {
        Ok(quota) => Ok(quota),
        Err(AgentAuthError::Unauthorized) => {
            auth = refresh_claude_token(ctx, &auth).await?;
            save_auth_file(&auth_path, &auth).await?;
            fetch_claude_quota(ctx, &auth).await
        }
        Err(err) => Err(err),
    }
}

async fn fetch_claude_quota(
    ctx: &AgentAuthContext,
    auth: &ClaudeTokenStorage,
) -> Result<AgentQuota, AgentAuthError> {
    let response = ctx
        .http_client()
        .await?
        .get(CLAUDE_USAGE_URL)
        .bearer_auth(&auth.access_token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("Accept", "application/json")
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    match status {
        ReqwestStatusCode::UNAUTHORIZED => return Err(AgentAuthError::Unauthorized),
        status if !status.is_success() => {
            warn!("Claude usage request failed: status {} body {}", status, body);
            return Err(AgentAuthError::Parse(format!(
                "Claude usage request failed ({}): {}",
                status, body
            )));
        }
        _ => {}
    }

    let data: ClaudeUsageResponse = serde_json::from_str(&body).map_err(|err| {
        let snippet: String = body.chars().take(200).collect();
        warn!(
            "Claude usage response decode failed: {} body {}",
            err, snippet
        );
        AgentAuthError::Parse(format!(
            "Claude usage response not JSON: {}",
            snippet
        ))
    })?;
    let mut entries = Vec::new();

    let five_hour_reset = data
        .five_hour
        .resets_at
        .as_deref()
        .and_then(parse_rfc3339_to_epoch);
    entries.push(AgentQuotaEntry {
        label: "5h".to_string(),
        used_percent: data.five_hour.utilization,
        reset_at: five_hour_reset,
    });

    let seven_day_reset = data
        .seven_day
        .resets_at
        .as_deref()
        .and_then(parse_rfc3339_to_epoch);
    entries.push(AgentQuotaEntry {
        label: "7d".to_string(),
        used_percent: data.seven_day.utilization,
        reset_at: seven_day_reset,
    });

    if let Some(window) = data.seven_day_sonnet.as_ref() {
        entries.push(AgentQuotaEntry {
            label: "7d sonnet".to_string(),
            used_percent: window.utilization,
            reset_at: window
                .resets_at
                .as_deref()
                .and_then(parse_rfc3339_to_epoch),
        });
    }
    if let Some(window) = data.seven_day_opus.as_ref() {
        entries.push(AgentQuotaEntry {
            label: "7d opus".to_string(),
            used_percent: window.utilization,
            reset_at: window
                .resets_at
                .as_deref()
                .and_then(parse_rfc3339_to_epoch),
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
    ctx: &AgentAuthContext,
    code: &str,
    code_verifier: &str,
    state: &str,
) -> Result<ClaudeTokenResponse, AgentAuthError> {
    let response = ctx
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
    ctx: &AgentAuthContext,
    auth: &ClaudeTokenStorage,
) -> Result<ClaudeTokenStorage, AgentAuthError> {
    let response = ctx
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

fn should_refresh_claude(auth: &ClaudeTokenStorage) -> bool {
    let expire = DateTime::parse_from_rfc3339(&auth.expire)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    expire - Utc::now() < ChronoDuration::minutes(5)
}
