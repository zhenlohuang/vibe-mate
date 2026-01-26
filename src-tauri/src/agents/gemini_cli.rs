use crate::agents::{
    auth::{
        auth_path_for_provider_id, build_google_auth_url, exchange_google_code,
        parse_google_id_token, refresh_google_token, save_auth_file, should_refresh_google,
        AgentAuthContext, AgentAuthError, AuthFlowStart,
    },
    binary_is_installed, resolve_binary_version, AgentMetadata, CodingAgentDefinition,
};
use crate::models::{AgentQuota, AgentType, Provider, ProviderStatus};

use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

const GEMINI_CLIENT_ID: &str =
    "681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com";
const GEMINI_CLIENT_SECRET: &str = "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";
const GEMINI_REDIRECT_URI: &str = "http://localhost:8085/oauth2callback";
const GEMINI_CALLBACK_PATH: &str = "/oauth2callback";
const GEMINI_CALLBACK_PORT: u16 = 8085;

const GEMINI_SCOPES: &[&str] = &[
    "openid",
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
];

pub struct GeminiCliAgent;

impl GeminiCliAgent {
    pub const METADATA: AgentMetadata = AgentMetadata {
        agent_type: AgentType::GeminiCLI,
        name: "Gemini CLI",
        binary: "gemini",
        default_config_file: "~/.gemini/settings.json",
        default_auth_file: "~/.gemini/credentials.json",
    };
}

impl CodingAgentDefinition for GeminiCliAgent {
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
struct GeminiTokenStorage {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub timestamp: i64,
    pub expire: String,
    pub email: String,
    pub project_id: Option<String>,
}

pub(crate) fn start_auth_flow(state: &str) -> Result<AuthFlowStart, AgentAuthError> {
    let auth_url = build_google_auth_url(
        GEMINI_CLIENT_ID,
        GEMINI_REDIRECT_URI,
        GEMINI_SCOPES,
        state,
    )?;
    Ok(AuthFlowStart {
        auth_url,
        callback_path: GEMINI_CALLBACK_PATH,
        callback_port: GEMINI_CALLBACK_PORT,
        code_verifier: String::new(),
    })
}

pub(crate) async fn complete_auth(
    ctx: &AgentAuthContext,
    provider_id: &str,
    _state: &str,
    code: &str,
    _code_verifier: &str,
) -> Result<(), AgentAuthError> {
    let token = exchange_google_code(
        ctx,
        code,
        GEMINI_CLIENT_ID,
        GEMINI_CLIENT_SECRET,
        GEMINI_REDIRECT_URI,
    )
    .await?;
    let access_token = token.access_token;
    let refresh_token = token
        .refresh_token
        .ok_or_else(|| AgentAuthError::Parse("Missing refresh_token".to_string()))?;
    let email = match token.id_token.as_deref() {
        Some(id_token) => match parse_google_id_token(id_token) {
            Ok(email) => email,
            Err(err) => {
                warn!("Failed to parse Google id_token: {}", err);
                ctx.fetch_google_email(&access_token).await?
            }
        },
        None => ctx.fetch_google_email(&access_token).await?,
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

    let auth_path = auth_path_for_provider_id(provider_id)?;
    info!("Saving auth token to {}", auth_path.display());
    save_auth_file(&auth_path, &storage).await?;
    ctx.update_provider_status(provider_id, ProviderStatus::Connected)
        .await?;

    Ok(())
}

pub(crate) async fn get_quota(
    ctx: &AgentAuthContext,
    provider: &Provider,
) -> Result<AgentQuota, AgentAuthError> {
    let (auth_path, mut auth): (std::path::PathBuf, GeminiTokenStorage) = ctx
        .load_and_normalize_auth(provider)
        .await?;

    if should_refresh_google(&auth.timestamp, auth.expires_in) {
        auth = refresh_gemini_token(ctx, &auth).await?;
        save_auth_file(&auth_path, &auth).await?;
    }

    fetch_gemini_quota(&auth).await
}

async fn refresh_gemini_token(
    ctx: &AgentAuthContext,
    auth: &GeminiTokenStorage,
) -> Result<GeminiTokenStorage, AgentAuthError> {
    let token = refresh_google_token(
        ctx,
        &auth.refresh_token,
        GEMINI_CLIENT_ID,
        GEMINI_CLIENT_SECRET,
    )
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

async fn fetch_gemini_quota(
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
