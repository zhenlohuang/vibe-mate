mod agents;
mod commands;
mod models;
mod services;
mod storage;

use std::sync::Arc;
use storage::{merge_coding_agents, ConfigStore};
use services::{
    AgentAuthService, AgentService, ConfigService, ProviderService, ProxyServer, RouterService,
};
use tauri::Manager;

/// Get config directory path (~/.vibemate/)
fn get_config_dir() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("Failed to get home directory");
    home.join(".vibemate")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("vibe_mate=debug".parse().unwrap())
                .add_directive("tower_http=debug".parse().unwrap()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Use ~/.vibemate/ as config directory
            let config_dir = get_config_dir();

            // Initialize unified config storage
            let store = Arc::new(ConfigStore::new(config_dir));
            tauri::async_runtime::block_on(async {
                store.init().await.expect("Failed to init storage");
            });

            // Initialize services
            let provider_service = Arc::new(ProviderService::new(store.clone()));
            let router_service = Arc::new(RouterService::new(store.clone()));
            let agent_service = Arc::new(AgentService::new());
            let config_service = Arc::new(ConfigService::new(store.clone()));
            let agent_auth_service = Arc::new(AgentAuthService::new(store.clone()));
            
            // Create the proxy server with access to the config store
            let proxy_server = Arc::new(ProxyServer::new(store.clone()));

            // Discover coding agents at startup and merge with stored config (cleans up removed agents)
            let store_clone = store.clone();
            let agent_service_clone = agent_service.clone();
            tauri::async_runtime::block_on(async move {
                match agent_service_clone.discover_agents().await {
                    Ok(discovered) => {
                        let config = store_clone.get_config().await;
                        let merged = merge_coding_agents(
                            &config.coding_agents,
                            discovered,
                            &config.dashboard.featured_agents,
                        );
                        if let Err(e) = store_clone.update(|c| c.coding_agents = merged).await {
                            tracing::warn!("Failed to save coding agents config: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to discover coding agents at startup: {}", e);
                    }
                }
            });

            // Register services to Tauri state management
            app.manage(store);
            app.manage(provider_service);
            app.manage(router_service);
            app.manage(agent_service);
            app.manage(config_service);
            app.manage(agent_auth_service);
            app.manage(proxy_server.clone());

            // Auto-start proxy server on port 12345
            let proxy_server_clone = proxy_server.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = proxy_server_clone.start(12345).await {
                    tracing::error!("Failed to start proxy server: {}", e);
                } else {
                    tracing::info!("Vibe Mate server started - OpenAI: /api/openai, Anthropic: /api/anthropic");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Provider commands
            commands::list_providers,
            commands::create_provider,
            commands::update_provider,
            commands::delete_provider,
            commands::test_connection,
            // Agent auth commands
            commands::start_agent_auth,
            commands::complete_agent_auth,
            commands::get_agent_quota,
            commands::list_agent_accounts,
            commands::remove_agent_auth,
            // Router commands
            commands::list_rules,
            commands::create_rule,
            commands::update_rule,
            commands::delete_rule,
            commands::reorder_rules,
            // Agent commands
            commands::check_status,
            commands::read_agent_config,
            commands::save_agent_config,
            // Config commands
            commands::get_config,
            commands::update_config,
            commands::test_latency,
            commands::get_coding_agents,
            commands::refresh_coding_agents,
            commands::set_coding_agent_featured,
            // System commands
            commands::proxy_status,
            commands::start_proxy,
            commands::stop_proxy,
            commands::get_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
