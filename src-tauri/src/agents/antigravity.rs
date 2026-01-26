use std::collections::HashMap;

use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};

use crate::agents::auth::{
    auth_path_for_provider_id, build_google_auth_url, exchange_google_code, parse_google_id_token,
    parse_rfc3339_to_epoch, refresh_google_token, save_auth_file, should_refresh_google,
    AgentAuthContext, AgentAuthError, AuthFlowStart,
};
use crate::models::{AgentQuota, AgentQuotaEntry, Provider, ProviderStatus};

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

const ANTIGRAVITY_SCOPES: &[&str] = &[
    "openid",
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
    "https://www.googleapis.com/auth/cclog",
    "https://www.googleapis.com/auth/experimentsandconfigs",
];

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

pub(crate) fn start_auth_flow(state: &str) -> Result<AuthFlowStart, AgentAuthError> {
    let auth_url = build_google_auth_url(
        ANTIGRAVITY_CLIENT_ID,
        ANTIGRAVITY_REDIRECT_URI,
        ANTIGRAVITY_SCOPES,
        state,
    )?;
    Ok(AuthFlowStart {
        auth_url,
        callback_path: ANTIGRAVITY_CALLBACK_PATH,
        callback_port: ANTIGRAVITY_CALLBACK_PORT,
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
        ANTIGRAVITY_CLIENT_ID,
        ANTIGRAVITY_CLIENT_SECRET,
        ANTIGRAVITY_REDIRECT_URI,
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
    let project_id = resolve_antigravity_project(ctx, &access_token).await?;

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
    let (auth_path, mut auth): (std::path::PathBuf, AntigravityTokenStorage) = ctx
        .load_and_normalize_auth(provider)
        .await?;

    if should_refresh_google(&auth.timestamp, auth.expires_in) {
        auth = refresh_antigravity_token(ctx, &auth).await?;
        save_auth_file(&auth_path, &auth).await?;
    }

    match fetch_antigravity_quota(ctx, &auth).await {
        Ok(quota) => Ok(quota),
        Err(AgentAuthError::Unauthorized) => {
            auth = refresh_antigravity_token(ctx, &auth).await?;
            save_auth_file(&auth_path, &auth).await?;
            fetch_antigravity_quota(ctx, &auth).await
        }
        Err(err) => Err(err),
    }
}

async fn refresh_antigravity_token(
    ctx: &AgentAuthContext,
    auth: &AntigravityTokenStorage,
) -> Result<AntigravityTokenStorage, AgentAuthError> {
    let token = refresh_google_token(
        ctx,
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

async fn fetch_antigravity_quota(
    ctx: &AgentAuthContext,
    auth: &AntigravityTokenStorage,
) -> Result<AgentQuota, AgentAuthError> {
    let mut body = json!({});
    if !auth.project_id.is_empty() {
        body["project"] = json!(auth.project_id.clone());
    }

    let response = ctx
        .http_client()
        .await?
        .post(ANTIGRAVITY_FETCH_MODELS_URL)
        .bearer_auth(&auth.access_token)
        .header("User-Agent", "antigravity/1.11.3 Darwin/arm64")
        .json(&body)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::UNAUTHORIZED => return Err(AgentAuthError::Unauthorized),
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

async fn resolve_antigravity_project(
    ctx: &AgentAuthContext,
    access_token: &str,
) -> Result<String, AgentAuthError> {
    let response = load_code_assist(ctx, access_token).await?;
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

    onboard_user(ctx, access_token, &tier_id).await
}

async fn load_code_assist(
    ctx: &AgentAuthContext,
    access_token: &str,
) -> Result<LoadCodeAssistResponse, AgentAuthError> {
    let response = ctx
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
    ctx: &AgentAuthContext,
    access_token: &str,
    tier_id: &str,
) -> Result<String, AgentAuthError> {
    for attempt in 1..=5 {
        let response = ctx
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

fn project_ref_to_id(project: ProjectRef) -> Option<String> {
    match project {
        ProjectRef::String(value) => Some(value),
        ProjectRef::Object { id } => Some(id),
    }
}
