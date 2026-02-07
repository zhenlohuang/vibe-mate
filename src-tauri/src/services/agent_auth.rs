use std::collections::HashMap;
use std::sync::Arc;

use futures_util::future::join_all;
use axum::{
    extract::{Query, State},
    http::StatusCode as AxumStatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::agents::auth::{auth_path_for_agent_type, read_email_from_auth, random_state};
use crate::agents::{
    complete_agent_auth, get_agent_quota, start_agent_auth_flow, AgentAuthContext, AgentAuthError,
};
use crate::models::{AgentAccountInfo, AgentAuthStart, AgentProviderType, AgentQuota};
use crate::storage::ConfigStore;

#[derive(Clone)]
struct AuthServerState {
    expected_state: String,
    sender: Arc<Mutex<Option<oneshot::Sender<AuthCallback>>>>,
}

#[derive(Debug)]
struct PendingAuth {
    agent_type: AgentProviderType,
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
    ctx: AgentAuthContext,
    pending: Arc<Mutex<HashMap<String, PendingAuth>>>,
}

impl AgentAuthService {
    pub fn new(store: Arc<ConfigStore>) -> Self {
        Self {
            ctx: AgentAuthContext::new(store),
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_auth(&self, agent_type: AgentProviderType) -> Result<AgentAuthStart, AgentAuthError> {
        info!("Starting agent auth flow for {:?}", agent_type);
        let mut pending = self.pending.lock().await;
        if !pending.is_empty() {
            warn!("Auth flow already in progress");
            return Err(AgentAuthError::FlowInProgress);
        }

        let flow_id = Uuid::new_v4().to_string();
        let state = random_state();
        let flow = start_agent_auth_flow(&agent_type, &state)?;

        let (code_tx, code_rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let server_state = AuthServerState {
            expected_state: state.clone(),
            sender: Arc::new(Mutex::new(Some(code_tx))),
        };

        let app = Router::new()
            .route(flow.callback_path, get(auth_callback))
            .with_state(server_state);

        let listener = TcpListener::bind(("127.0.0.1", flow.callback_port)).await?;
        info!(
            "Auth callback server listening on 127.0.0.1:{}{}",
            flow.callback_port, flow.callback_path
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
                agent_type,
                state,
                code_verifier: flow.code_verifier,
                receiver: Some(code_rx),
                shutdown: Some(shutdown_tx),
            },
        );

        Ok(AgentAuthStart {
            flow_id,
            auth_url: flow.auth_url,
        })
    }

    pub async fn complete_auth(&self, flow_id: &str) -> Result<AgentAccountInfo, AgentAuthError> {
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
        complete_agent_auth(
            &self.ctx,
            &pending.agent_type,
            &pending.state,
            &callback.code,
            &pending.code_verifier,
        )
        .await?;

        let email = read_email_from_auth(&pending.agent_type).await;
        Ok(AgentAccountInfo {
            agent_type: pending.agent_type,
            is_authenticated: true,
            email,
        })
    }

    pub async fn get_quota(&self, agent_type: AgentProviderType) -> Result<AgentQuota, AgentAuthError> {
        get_agent_quota(&self.ctx, &agent_type).await
    }

    pub async fn list_accounts(&self) -> Vec<AgentAccountInfo> {
        let variants = [
            AgentProviderType::Codex,
            AgentProviderType::ClaudeCode,
            AgentProviderType::GeminiCli,
            AgentProviderType::Antigravity,
        ];
        let results = join_all(variants.iter().map(|agent_type| {
            let agent_type = agent_type.clone();
            async move {
            let path = match auth_path_for_agent_type(&agent_type) {
                Ok(p) => p,
                Err(_) => {
                    return AgentAccountInfo {
                        agent_type,
                        is_authenticated: false,
                        email: None,
                    };
                }
            };
            let is_authenticated = path.exists();
            let email = if is_authenticated {
                read_email_from_auth(&agent_type).await
            } else {
                None
            };
            AgentAccountInfo {
                agent_type,
                is_authenticated,
                email,
            }
            }
        }))
        .await;
        results
    }

    pub async fn remove_auth(&self, agent_type: &AgentProviderType) -> Result<(), AgentAuthError> {
        let path = auth_path_for_agent_type(agent_type)?;
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
            info!("Removed auth file for {:?}: {}", agent_type, path.display());
        }
        Ok(())
    }
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
