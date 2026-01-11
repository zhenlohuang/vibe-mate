use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{header, Method, Request, Response, StatusCode},
    routing::any,
    Router,
};
use bytes::Bytes;
use futures_util::StreamExt;
use glob::Pattern;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, RwLock};
use tower_http::cors::{Any, CorsLayer};

use crate::models::{ApiGroup, Provider, RoutingRule, RuleType, VibeMateConfig};
use crate::storage::ConfigStore;

/// Create HTTP client with proxy support based on config
fn create_http_client(config: &VibeMateConfig) -> Client {
    let mut builder = Client::builder().timeout(std::time::Duration::from_secs(300));

    if config.app.enable_proxy {
        if let (Some(host), Some(port)) = (&config.app.proxy_host, config.app.proxy_port) {
            let proxy_url = format!("http://{}:{}", host, port);
            tracing::info!("Creating HTTP client with proxy: {}", proxy_url);

            match reqwest::Proxy::all(&proxy_url) {
                Ok(mut proxy) => {
                    // Configure no_proxy list
                    if !config.app.no_proxy.is_empty() {
                        tracing::debug!("Configuring no_proxy patterns: {:?}", config.app.no_proxy);
                        let no_proxy = reqwest::NoProxy::from_string(&config.app.no_proxy.join(","));
                        proxy = proxy.no_proxy(no_proxy);
                    }
                    builder = builder.proxy(proxy);
                }
                Err(e) => {
                    tracing::error!("Failed to create proxy: {}", e);
                    builder = builder.no_proxy();
                }
            }
        } else {
            tracing::warn!("Proxy enabled but host/port not configured");
            builder = builder.no_proxy();
        }
    } else {
        tracing::debug!("Proxy disabled, creating client without proxy");
        builder = builder.no_proxy();
    }

    builder.build().expect("Failed to create HTTP client")
}

/// Proxy server state shared across the application
pub struct ProxyServer {
    is_running: AtomicBool,
    port: AtomicU64,
    request_count: AtomicU64,
    store: Arc<ConfigStore>,
    shutdown_tx: RwLock<Option<oneshot::Sender<()>>>,
}

impl ProxyServer {
    pub fn new(store: Arc<ConfigStore>) -> Self {
        Self {
            is_running: AtomicBool::new(false),
            port: AtomicU64::new(12345),
            request_count: AtomicU64::new(0),
            store,
            shutdown_tx: RwLock::new(None),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn port(&self) -> u16 {
        self.port.load(Ordering::SeqCst) as u16
    }

    pub fn request_count(&self) -> u64 {
        self.request_count.load(Ordering::SeqCst)
    }

    pub fn increment_request_count(&self) {
        self.request_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Start the proxy server on the given port
    pub async fn start(self: &Arc<Self>, port: u16) -> Result<(), ProxyError> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(ProxyError::AlreadyRunning);
        }

        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        *self.shutdown_tx.write().await = Some(shutdown_tx);

        // Create HTTP client based on global proxy settings
        let config = self.store.get_config().await;
        let http_client = create_http_client(&config);

        // Setup CORS
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(Any);

        // Build the router
        let app_state = AppState {
            server: Arc::clone(self),
            http_client,
        };

        let app = Router::new()
            .route("/", any(health_check))
            .route("/health", any(health_check))
            .route("/api/openai/{*path}", any(openai_proxy_handler))
            .route("/api/anthropic/{*path}", any(anthropic_proxy_handler))
            .route("/api/{*path}", any(generic_proxy_handler))
            .layer(cors)
            .with_state(app_state);

        // Bind to the address
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| ProxyError::BindFailed(format!("Failed to bind to {}: {}", addr, e)))?;

        self.port.store(port as u64, Ordering::SeqCst);
        self.is_running.store(true, Ordering::SeqCst);

        tracing::info!("Vibe Mate server started on http://{}", addr);

        // Run the server with graceful shutdown
        let server_handle = self.clone();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .ok();

            server_handle.is_running.store(false, Ordering::SeqCst);
            tracing::info!("Proxy server stopped");
        });

        Ok(())
    }

    /// Stop the proxy server
    pub async fn stop(&self) -> Result<(), ProxyError> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(ProxyError::NotRunning);
        }

        if let Some(tx) = self.shutdown_tx.write().await.take() {
            let _ = tx.send(());
        }

        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Get the config store
    pub fn config_store(&self) -> &Arc<ConfigStore> {
        &self.store
    }
}

#[derive(Clone)]
struct AppState {
    server: Arc<ProxyServer>,
    /// HTTP client with global proxy settings
    http_client: Client,
}

fn should_skip_request_header(name: &header::HeaderName) -> bool {
    matches!(
        name,
        &header::HOST
            | &header::AUTHORIZATION
            | &header::CONTENT_LENGTH
            | &header::TRANSFER_ENCODING
            | &header::CONNECTION
            | &header::PROXY_AUTHORIZATION
    )
}

/// Health check endpoint
async fn health_check() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"status":"ok"}"#))
        .unwrap()
}

/// Generic API proxy handler (for /api/*)
async fn generic_proxy_handler(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    state.server.increment_request_count();

    // Get the path from the request and strip the /api prefix
    let full_path = req.uri().path().to_string();
    let path = full_path
        .strip_prefix("/api")
        .unwrap_or(&full_path)
        .to_string();
    let method = req.method().clone();

    tracing::debug!(
        "Generic proxy request: {} {} (original: {})",
        method,
        path,
        full_path
    );

    // Read the request body
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read request body: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Extract model from request body (for chat/completions requests)
    let model_name = extract_model_from_body(&body_bytes);

    tracing::debug!("Request model: {:?}", model_name);

    // Get config and find the matching provider
    let config = state.server.config_store().get_config().await;

    let resolved = match resolve_provider(
        &config,
        ApiGroup::Generic,
        &full_path,
        model_name.as_deref(),
    ) {
        Some(r) => r,
        None => {
            tracing::error!("No provider found for model: {:?}", model_name);
            return Ok(error_response(
                StatusCode::BAD_GATEWAY,
                "No provider configured. Please add a provider in Vibe Mate settings.",
            ));
        }
    };

    tracing::info!(
        "Routing to provider: {} ({}), model: {} -> {}",
        resolved.provider.name,
        resolved.provider.api_base_url,
        model_name.as_deref().unwrap_or("unknown"),
        resolved.final_model
    );

    // Build the target URL - handle the case where api_base_url already contains /v1
    let base_url = resolved.provider.api_base_url.trim_end_matches('/');
    let target_url = if base_url.ends_with("/v1") && path.starts_with("/v1") {
        // If base URL ends with /v1 and path starts with /v1, strip /v1 from path
        format!("{}{}", base_url, &path[3..])
    } else {
        format!("{}{}", base_url, path)
    };

    // Prepare the request body (potentially rewrite the model)
    let final_body = if resolved.model_rewritten {
        rewrite_model_in_body(&body_bytes, &resolved.final_model)
    } else {
        body_bytes.to_vec()
    };

    // Select HTTP client based on provider's enable_proxy setting
    let http_client = &state.http_client;

    // Build the outgoing request
    let mut outgoing_req = http_client.request(method.clone(), &target_url);

    // Copy headers, but replace Authorization and Host
    for (key, value) in parts.headers.iter() {
        if should_skip_request_header(key) {
            continue;
        }
        if let Ok(v) = value.to_str() {
            outgoing_req = outgoing_req.header(key.as_str(), v);
        }
    }

    // Add the API key based on provider type
    outgoing_req = add_auth_header(outgoing_req, &resolved.provider);

    // Set content type and body
    outgoing_req = outgoing_req
        .header(header::CONTENT_TYPE, "application/json")
        .body(final_body);

    // Send the request
    tracing::debug!("Sending request to: {}", target_url);
    let response = match outgoing_req.send().await {
        Ok(resp) => {
            tracing::info!("Received response: {} from {}", resp.status(), target_url);
            resp
        }
        Err(e) => {
            tracing::error!("Failed to forward request to {}: {}", target_url, e);
            return Ok(error_response(
                StatusCode::BAD_GATEWAY,
                &format!("Failed to connect to provider: {}", e),
            ));
        }
    };

    // Check if it's a streaming response
    let is_streaming = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/event-stream"))
        .unwrap_or(false);

    if is_streaming {
        // Handle streaming response
        handle_streaming_response(response).await
    } else {
        // Handle regular response
        handle_regular_response(response).await
    }
}

/// OpenAI compatible API proxy handler
async fn openai_proxy_handler(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    state.server.increment_request_count();

    // Get the path from the request and strip the /api/openai prefix
    let full_path = req.uri().path().to_string();
    let path = full_path
        .strip_prefix("/api/openai")
        .unwrap_or(&full_path)
        .to_string();
    let method = req.method().clone();

    tracing::debug!(
        "OpenAI proxy request: {} {} (original: {})",
        method,
        path,
        full_path
    );

    // Read the request body
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read request body: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Extract model from request body (for chat/completions requests)
    let model_name = extract_model_from_body(&body_bytes);

    tracing::debug!("Request model: {:?}", model_name);

    // Get config and find the matching provider
    let config = state.server.config_store().get_config().await;

    let resolved =
        match resolve_provider(&config, ApiGroup::OpenAI, &full_path, model_name.as_deref()) {
            Some(r) => r,
            None => {
                tracing::error!("No provider found for model: {:?}", model_name);
                return Ok(error_response(
                    StatusCode::BAD_GATEWAY,
                    "No provider configured. Please add a provider in Vibe Mate settings.",
                ));
            }
        };

    tracing::info!(
        "Routing to provider: {} ({}), model: {} -> {}",
        resolved.provider.name,
        resolved.provider.api_base_url,
        model_name.as_deref().unwrap_or("unknown"),
        resolved.final_model
    );

    // Build the target URL - handle the case where api_base_url already contains /v1
    let base_url = resolved.provider.api_base_url.trim_end_matches('/');
    let target_url = if base_url.ends_with("/v1") && path.starts_with("/v1") {
        // If base URL ends with /v1 and path starts with /v1, strip /v1 from path
        format!("{}{}", base_url, &path[3..])
    } else {
        format!("{}{}", base_url, path)
    };

    // Prepare the request body (potentially rewrite the model)
    let final_body = if resolved.model_rewritten {
        rewrite_model_in_body(&body_bytes, &resolved.final_model)
    } else {
        body_bytes.to_vec()
    };

    // Select HTTP client based on provider's enable_proxy setting
    let http_client = &state.http_client;

    // Build the outgoing request
    let mut outgoing_req = http_client.request(method.clone(), &target_url);

    // Copy headers, but replace Authorization and Host
    for (key, value) in parts.headers.iter() {
        if should_skip_request_header(key) {
            continue;
        }
        if let Ok(v) = value.to_str() {
            outgoing_req = outgoing_req.header(key.as_str(), v);
        }
    }

    // Add the API key based on provider type
    outgoing_req = add_auth_header(outgoing_req, &resolved.provider);

    // Set content type and body
    outgoing_req = outgoing_req
        .header(header::CONTENT_TYPE, "application/json")
        .body(final_body);

    // Send the request
    tracing::debug!("Sending request to: {}", target_url);
    let response = match outgoing_req.send().await {
        Ok(resp) => {
            tracing::info!("Received response: {} from {}", resp.status(), target_url);
            resp
        }
        Err(e) => {
            tracing::error!("Failed to forward request to {}: {}", target_url, e);
            return Ok(error_response(
                StatusCode::BAD_GATEWAY,
                &format!("Failed to connect to provider: {}", e),
            ));
        }
    };

    // Check if it's a streaming response
    let is_streaming = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/event-stream"))
        .unwrap_or(false);

    if is_streaming {
        // Handle streaming response
        handle_streaming_response(response).await
    } else {
        // Handle regular response
        handle_regular_response(response).await
    }
}

/// Anthropic API proxy handler
async fn anthropic_proxy_handler(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    state.server.increment_request_count();

    // Get the path from the request and strip the /api/anthropic prefix
    let full_path = req.uri().path().to_string();
    let path = full_path
        .strip_prefix("/api/anthropic")
        .unwrap_or(&full_path)
        .to_string();
    let method = req.method().clone();

    tracing::debug!(
        "Anthropic proxy request: {} {} (original: {})",
        method,
        path,
        full_path
    );

    // Read the request body
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to read request body: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Extract model from request body
    let model_name = extract_model_from_body(&body_bytes);

    tracing::debug!("Request model: {:?}", model_name);

    // Get config and find the matching provider
    let config = state.server.config_store().get_config().await;

    let resolved = match resolve_provider(
        &config,
        ApiGroup::Anthropic,
        &full_path,
        model_name.as_deref(),
    ) {
        Some(r) => r,
        None => {
            tracing::error!("No provider found for model: {:?}", model_name);
            return Ok(error_response(
                StatusCode::BAD_GATEWAY,
                "No provider configured. Please add a provider in Vibe Mate settings.",
            ));
        }
    };

    tracing::info!(
        "Routing to provider: {} ({}), model: {} -> {}",
        resolved.provider.name,
        resolved.provider.api_base_url,
        model_name.as_deref().unwrap_or("unknown"),
        resolved.final_model
    );

    // Build the target URL for Anthropic
    let base_url = resolved.provider.api_base_url.trim_end_matches('/');
    let target_url = format!("{}{}", base_url, path);

    // Prepare the request body (potentially rewrite the model)
    let final_body = if resolved.model_rewritten {
        rewrite_model_in_body(&body_bytes, &resolved.final_model)
    } else {
        body_bytes.to_vec()
    };

    // Select HTTP client based on provider's enable_proxy setting
    let http_client = &state.http_client;

    // Build the outgoing request
    let mut outgoing_req = http_client.request(method.clone(), &target_url);

    // Copy headers, but replace Authorization and Host
    for (key, value) in parts.headers.iter() {
        if should_skip_request_header(key) {
            continue;
        }
        if let Ok(v) = value.to_str() {
            outgoing_req = outgoing_req.header(key.as_str(), v);
        }
    }

    // Add the API key based on provider type
    outgoing_req = add_auth_header(outgoing_req, &resolved.provider);

    // Set content type and body
    outgoing_req = outgoing_req
        .header(header::CONTENT_TYPE, "application/json")
        .body(final_body);

    // Send the request
    tracing::debug!("Sending request to: {}", target_url);
    let response = match outgoing_req.send().await {
        Ok(resp) => {
            tracing::info!("Received response: {} from {}", resp.status(), target_url);
            resp
        }
        Err(e) => {
            tracing::error!("Failed to forward request to {}: {}", target_url, e);
            return Ok(error_response(
                StatusCode::BAD_GATEWAY,
                &format!("Failed to connect to provider: {}", e),
            ));
        }
    };

    // Check if it's a streaming response
    let is_streaming = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/event-stream"))
        .unwrap_or(false);

    if is_streaming {
        // Handle streaming response
        handle_streaming_response(response).await
    } else {
        // Handle regular response
        handle_regular_response(response).await
    }
}

/// Resolved provider information
struct ResolvedProvider {
    provider: Provider,
    final_model: String,
    model_rewritten: bool,
}

/// Resolve which provider to use based on routing rules and model name
fn resolve_provider(
    config: &VibeMateConfig,
    api_group: ApiGroup,
    request_path: &str,
    model_name: Option<&str>,
) -> Option<ResolvedProvider> {
    // If there are no providers, return None
    if config.providers.is_empty() {
        return None;
    }

    // Get enabled routing rules sorted by priority
    let mut rules: Vec<&RoutingRule> = config.routing_rules.iter().filter(|r| r.enabled).collect();
    rules.sort_by_key(|r| r.priority);

    let rule = match_rule_for_group(&rules, &api_group, request_path, model_name).or_else(|| {
        if api_group == ApiGroup::Generic {
            None
        } else {
            match_rule_for_group(&rules, &ApiGroup::Generic, request_path, model_name)
        }
    });

    if let Some(rule) = rule {
        if let Some(provider) = config.providers.iter().find(|p| p.id == rule.provider_id) {
            let final_model = model_name
                .map(|model| {
                    rule.model_rewrite
                        .clone()
                        .unwrap_or_else(|| model.to_string())
                })
                .unwrap_or_default();
            return Some(ResolvedProvider {
                provider: provider.clone(),
                final_model,
                model_rewritten: rule.model_rewrite.is_some() && model_name.is_some(),
            });
        }
    }

    // Fall back to default provider
    let default_provider = config
        .providers
        .iter()
        .find(|p| p.is_default)
        .or_else(|| config.providers.first())?;

    Some(ResolvedProvider {
        provider: default_provider.clone(),
        final_model: model_name.unwrap_or("").to_string(),
        model_rewritten: false,
    })
}

fn match_rule_for_group<'a>(
    rules: &'a [&RoutingRule],
    api_group: &ApiGroup,
    request_path: &str,
    model_name: Option<&str>,
) -> Option<&'a RoutingRule> {
    let mut model_rules: Vec<&RoutingRule> = rules
        .iter()
        .copied()
        .filter(|r| &r.api_group == api_group && r.rule_type == RuleType::Model)
        .collect();
    model_rules.sort_by_key(|r| r.priority);

    if let Some(model) = model_name {
        for rule in model_rules {
            if matches_pattern(&rule.match_pattern, model) {
                return Some(rule);
            }
        }
    }

    let mut path_rules: Vec<&RoutingRule> = rules
        .iter()
        .copied()
        .filter(|r| &r.api_group == api_group && r.rule_type == RuleType::Path)
        .collect();
    if *api_group == ApiGroup::Generic {
        path_rules.sort_by_key(|r| (r.match_pattern == "/api/*", r.priority));
    } else {
        path_rules.sort_by_key(|r| r.priority);
    }

    for rule in path_rules {
        if matches_pattern(&rule.match_pattern, request_path) {
            return Some(rule);
        }
    }

    None
}

/// Match a pattern against a model name using glob-style matching
fn matches_pattern(pattern: &str, model_name: &str) -> bool {
    Pattern::new(pattern)
        .map(|p| p.matches(model_name))
        .unwrap_or(false)
}

/// Extract model name from request body
fn extract_model_from_body(body: &Bytes) -> Option<String> {
    #[derive(Deserialize)]
    struct RequestBody {
        model: Option<String>,
    }

    serde_json::from_slice::<RequestBody>(body)
        .ok()
        .and_then(|r| r.model)
}

/// Rewrite the model field in the request body
fn rewrite_model_in_body(body: &Bytes, new_model: &str) -> Vec<u8> {
    // Parse as JSON value, modify model, serialize back
    if let Ok(mut json) = serde_json::from_slice::<serde_json::Value>(body) {
        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                "model".to_string(),
                serde_json::Value::String(new_model.to_string()),
            );
        }
        serde_json::to_vec(&json).unwrap_or_else(|_| body.to_vec())
    } else {
        body.to_vec()
    }
}

/// Add authentication header based on provider type
fn add_auth_header(req: reqwest::RequestBuilder, provider: &Provider) -> reqwest::RequestBuilder {
    use crate::models::ProviderType;

    match provider.provider_type {
        ProviderType::Anthropic => {
            // Anthropic uses x-api-key header
            req.header("x-api-key", &provider.api_key)
                .header("anthropic-version", "2023-06-01")
        }
        ProviderType::Google => {
            // Google uses API key in URL or header
            req.header("x-goog-api-key", &provider.api_key)
        }
        _ => {
            // OpenAI, Azure, Custom use Bearer token
            req.header(
                header::AUTHORIZATION,
                format!("Bearer {}", provider.api_key),
            )
        }
    }
}

/// Handle regular (non-streaming) response
async fn handle_regular_response(
    response: reqwest::Response,
) -> Result<Response<Body>, StatusCode> {
    let status = response.status();
    let headers = response.headers().clone();

    let body_bytes = response.bytes().await.map_err(|e| {
        tracing::error!("Failed to read response body: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    tracing::debug!("Response body size: {} bytes", body_bytes.len());

    let mut builder = Response::builder().status(status);

    // Copy relevant headers (skip transfer-encoding as we're using a known body length)
    for (key, value) in headers.iter() {
        if key != header::TRANSFER_ENCODING {
            builder = builder.header(key, value);
        }
    }

    builder.body(Body::from(body_bytes)).map_err(|e| {
        tracing::error!("Failed to build response: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// Handle streaming (SSE) response
async fn handle_streaming_response(
    response: reqwest::Response,
) -> Result<Response<Body>, StatusCode> {
    let status = response.status();
    let headers = response.headers().clone();

    // Create a stream from the response body
    let stream = response.bytes_stream().map(|result| {
        result.map_err(|e| {
            tracing::error!("Streaming error: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })
    });

    let body = Body::from_stream(stream);

    let mut builder = Response::builder().status(status);

    for (key, value) in headers.iter() {
        builder = builder.header(key, value);
    }

    builder
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Create an error response
fn error_response(status: StatusCode, message: &str) -> Response<Body> {
    #[derive(Serialize)]
    struct ErrorResponse {
        error: ErrorDetail,
    }

    #[derive(Serialize)]
    struct ErrorDetail {
        message: String,
        #[serde(rename = "type")]
        error_type: String,
    }

    let error = ErrorResponse {
        error: ErrorDetail {
            message: message.to_string(),
            error_type: "proxy_error".to_string(),
        },
    };

    let body = serde_json::to_string(&error).unwrap_or_else(|_| message.to_string());

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()
        })
}

/// Proxy server errors
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("Proxy server is already running")]
    AlreadyRunning,
    #[error("Proxy server is not running")]
    NotRunning,
    #[error("Failed to bind: {0}")]
    BindFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
