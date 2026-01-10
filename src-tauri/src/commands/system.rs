use std::sync::Arc;
use std::time::Duration;
use tauri::State;

use crate::models::ProxyStatus;
use crate::services::ProxyServer;

#[tauri::command]
pub async fn proxy_status(
    state: State<'_, Arc<ProxyServer>>,
) -> Result<ProxyStatus, String> {
    let port = state.port();
    let request_count = state.request_count();
    
    // Actually check if the server is responding by calling health endpoint
    let is_running = if state.is_running() {
        check_health(port).await
    } else {
        false
    };
    
    Ok(ProxyStatus {
        is_running,
        port,
        request_count,
    })
}

/// Check if the proxy server is actually responding
async fn check_health(port: u16) -> bool {
    // Use no_proxy() to bypass system proxy for localhost health check
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .no_proxy()
        .build()
        .ok();
    
    if let Some(client) = client {
        let url = format!("http://127.0.0.1:{}/health", port);
        client.get(&url).send().await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    } else {
        false
    }
}

#[tauri::command]
pub async fn start_proxy(
    state: State<'_, Arc<ProxyServer>>,
) -> Result<(), String> {
    // Get the configured port (default 12345)
    let port = state.port();
    
    state.start(port).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_proxy(
    state: State<'_, Arc<ProxyServer>>,
) -> Result<(), String> {
    state.stop().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

