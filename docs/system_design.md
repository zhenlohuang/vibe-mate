# Vibe Mate - System Design Document

## 1. System Overview

Vibe Mate is a desktop companion application designed specifically for the "Vibe Coding" workflow, used to manage AI proxies and coding agents (such as Claude Code). The application adopts a modern Cyberpunk Minimalist design style, providing core functionalities including model provider management, model routing, coding agent monitoring, and network proxy configuration.

---

## 2. Technology Stack

### 2.1 Frontend Technology Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| TypeScript | ^5.7 | Type-safe JavaScript superset |
| Next.js | ^15.0 | React full-stack framework with SSR/SSG support |
| React | ^19.0 | UI component library |
| Shadcn/UI | latest | Component system based on Radix UI |
| Tailwind CSS | ^4.0 | Atomic CSS framework |
| Zustand | ^5.0 | Lightweight state management |
| TanStack Query | ^5.64 | Server state management and caching |
| Motion | ^12.0 | Animation library (formerly Framer Motion) |
| Lucide React | latest | Icon library |

### 2.2 Backend Technology Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| Rust | ^1.84 | Systems programming language |
| Tauri | ^2.2 | Cross-platform desktop application framework |
| Axum | ^0.8 | Async web framework |
| Tokio | ^1.43 | Async runtime |
| Serde | ^1.0 | Serialization/Deserialization |
| Serde JSON | ^1.0 | JSON file storage |
| dirs | ^6.0 | Get user home directory |
| Tower | ^0.5 | Service middleware |

---

## 3. System Architecture

### 3.1 Overall Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              Vibe Mate Desktop App                       │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                     Frontend (Next.js + React)                   │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │    │
│  │  │ General  │  │ Provider │  │  Router  │  │  Coding Agents   │ │    │
│  │  │   Page   │  │   Page   │  │   Page   │  │      Page        │ │    │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │    │
│  │  ┌──────────────────────────────────────────────────────────────┤ │    │
│  │  │              Shadcn/UI Component Library                     │ │    │
│  │  └──────────────────────────────────────────────────────────────┘ │    │
│  │  ┌──────────────────────────────────────────────────────────────┤ │    │
│  │  │       State Management (Zustand + TanStack Query)            │ │    │
│  │  └──────────────────────────────────────────────────────────────┘ │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                    │                                      │
│                          Tauri IPC Bridge                                │
│                                    │                                      │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      Backend (Rust + Axum)                       │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │    │
│  │  │   Provider   │  │    Router    │  │     Agent Service    │  │    │
│  │  │   Service    │  │   Service    │  │                      │  │    │
│  │  └──────────────┘  └──────────────┘  └──────────────────────┘  │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │    │
│  │  │   Network    │  │    Proxy     │  │   Config Service     │  │    │
│  │  │   Service    │  │   Server     │  │                      │  │    │
│  │  └──────────────┘  └──────────────┘  └──────────────────────┘  │    │
│  │  ┌──────────────────────────────────────────────────────────┐  │    │
│  │  │           JSON File Storage (Serde JSON)                  │  │    │
│  │  └──────────────────────────────────────────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
              ┌──────────────────────────────────────┐
              │         External AI Providers         │
              │  ┌──────────┐  ┌──────────┐          │
              │  │  OpenAI  │  │Anthropic │  ...     │
              │  └──────────┘  └──────────┘          │
              └──────────────────────────────────────┘
```

### 3.2 Architecture Layer Description

| Layer | Responsibility | Technology Implementation |
|-------|---------------|---------------------------|
| Presentation Layer | UI rendering, user interaction, state display | Next.js + Shadcn/UI |
| State Layer | Frontend state management, caching, data synchronization | Zustand + TanStack Query |
| Bridge Layer | Frontend-backend communication, IPC calls | Tauri IPC |
| Service Layer | Business logic processing, API proxy | Axum + Rust Services |
| Data Layer | Data persistence, configuration storage | JSON Files + Serde |

---

## 4. Module Design

### 4.1 Module Structure

```
vibe-mate/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Application entry point
│   │   ├── lib.rs                # Library entry point
│   │   ├── commands/             # Tauri IPC commands
│   │   │   ├── mod.rs
│   │   │   ├── provider.rs       # Provider-related commands
│   │   │   ├── router.rs         # Router-related commands
│   │   │   ├── agent.rs          # Coding agent-related commands
│   │   │   └── network.rs        # Network-related commands
│   │   ├── services/             # Business services
│   │   │   ├── mod.rs
│   │   │   ├── provider.rs       # Provider service
│   │   │   ├── router.rs         # Model routing service
│   │   │   ├── agent.rs          # Coding agent service (auto-discovery)
│   │   │   ├── proxy.rs          # Proxy server
│   │   │   └── network.rs        # Network service
│   │   ├── models/               # Data models
│   │   │   ├── mod.rs
│   │   │   ├── provider.rs
│   │   │   ├── routing_rule.rs
│   │   │   ├── agent.rs
│   │   │   └── network_config.rs
│   │   ├── storage/              # Configuration storage
│   │   │   ├── mod.rs
│   │   │   └── config_store.rs   # Unified configuration storage implementation
│   │   └── utils/                # Utility functions
│   │       ├── mod.rs
│   │       ├── crypto.rs         # Encryption utilities
│   │       └── error.rs          # Error handling
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                          # Next.js frontend
│   ├── app/                      # App Router
│   │   ├── layout.tsx            # Root layout
│   │   ├── page.tsx              # Home page (redirects to general)
│   │   ├── general/              # General settings page
│   │   │   └── page.tsx
│   │   ├── providers/            # Model provider page
│   │   │   └── page.tsx
│   │   ├── router/               # Model router page
│   │   │   └── page.tsx
│   │   ├── agents/               # Coding agents page
│   │   │   └── page.tsx
│   │   └── network/              # Network settings page
│   │       └── page.tsx
│   ├── components/               # Components
│   │   ├── ui/                   # Shadcn/UI components
│   │   ├── layout/               # Layout components
│   │   │   ├── sidebar.tsx
│   │   │   └── main-content.tsx
│   │   ├── providers/            # Provider-related components
│   │   │   ├── provider-card.tsx
│   │   │   └── provider-form.tsx
│   │   ├── router/               # Router-related components
│   │   │   ├── routing-rule-item.tsx
│   │   │   ├── routing-rule-list.tsx
│   │   │   └── fallback-section.tsx
│   │   ├── agents/               # Agent-related components
│   │   │   ├── agent-card.tsx
│   │   │   └── agent-status.tsx
│   │   └── network/              # Network-related components
│   │       └── network-form.tsx
│   ├── hooks/                    # Custom Hooks
│   │   ├── use-providers.ts
│   │   ├── use-routing-rules.ts
│   │   ├── use-agents.ts
│   │   └── use-tauri.ts
│   ├── stores/                   # Zustand Stores
│   │   ├── app-store.ts
│   │   ├── provider-store.ts
│   │   └── router-store.ts
│   ├── lib/                      # Utility library
│   │   ├── tauri.ts              # Tauri API wrapper
│   │   ├── utils.ts              # General utilities
│   │   └── constants.ts          # Constants definition
│   └── types/                    # TypeScript types
│       ├── provider.ts
│       ├── router.ts
│       ├── agent.ts
│       └── network.ts
│
├── public/                       # Static assets
├── docs/                         # Documentation
├── package.json
├── tailwind.config.ts
├── tsconfig.json
└── next.config.js
```

### 4.2 Core Module Description

#### 4.2.1 Provider Service

**Responsibility**: Manage AI model provider configurations and connection status

```rust
// src-tauri/src/services/provider.rs
pub struct ProviderService {
    store: Arc<ConfigStore>,
}

impl ProviderService {
    pub async fn list_providers(&self) -> Result<Vec<Provider>>;
    pub async fn create_provider(&self, input: CreateProviderInput) -> Result<Provider>;
    pub async fn update_provider(&self, id: String, input: UpdateProviderInput) -> Result<Provider>;
    pub async fn delete_provider(&self, id: String) -> Result<()>;
    pub async fn set_default_provider(&self, id: String) -> Result<()>;
    pub async fn test_connection(&self, id: String) -> Result<ConnectionStatus>;
}
```

#### 4.2.2 Router Service

**Responsibility**: Manage model routing rules and implement request distribution logic

```rust
// src-tauri/src/services/router.rs
pub struct RouterService {
    store: Arc<ConfigStore>,
    provider_service: Arc<ProviderService>,
}

impl RouterService {
    pub async fn list_rules(&self) -> Result<Vec<RoutingRule>>;
    pub async fn create_rule(&self, input: CreateRuleInput) -> Result<RoutingRule>;
    pub async fn update_rule(&self, id: String, input: UpdateRuleInput) -> Result<RoutingRule>;
    pub async fn delete_rule(&self, id: String) -> Result<()>;
    pub async fn reorder_rules(&self, rule_ids: Vec<String>) -> Result<()>;
    pub async fn match_provider(&self, model_name: &str) -> Result<ResolvedProvider>;
}
```

#### 4.2.3 Proxy Server

**Responsibility**: Run local proxy server, intercept and forward AI API requests

```rust
// src-tauri/src/services/proxy.rs
pub struct ProxyServer {
    router_service: Arc<RouterService>,
    config: ProxyConfig,
}

impl ProxyServer {
    pub async fn start(&self, port: u16) -> Result<()>;
    pub async fn stop(&self) -> Result<()>;
    pub fn status(&self) -> ProxyStatus;
    async fn handle_request(&self, req: Request) -> Response;
}
```

#### 4.2.4 Agent Service

**Responsibility**: Auto-discover coding agents installed on the system (such as Claude Code, Gemini CLI), detect their installation and authentication status

```rust
// src-tauri/src/services/agent.rs
pub struct AgentService;

impl AgentService {
    pub fn new() -> Self {
        Self
    }
    
    /// Discover all supported coding agents in the system
    pub async fn discover_agents(&self) -> Result<Vec<CodingAgent>>;
    
    /// Check if a specific agent is installed
    pub async fn is_installed(&self, agent_type: AgentType) -> bool;
    
    /// Check the authentication status of a specific agent
    pub async fn check_auth_status(&self, agent_type: AgentType) -> AgentStatus;
    
    /// Get version information for an agent
    pub async fn get_version(&self, agent_type: AgentType) -> Option<String>;
    
    /// Open the login flow for an agent
    pub async fn open_login(&self, agent_type: AgentType) -> Result<()>;
}

impl AgentService {
    /// Detect installation status by checking if the command exists
    async fn detect_installation(&self, agent_type: AgentType) -> bool {
        let command = agent_type.detection_command();
        // Use `which` (Unix) or `where` (Windows) to detect if command exists
        std::process::Command::new("which")
            .arg(command)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
```

---

## 5. Data Model Design

### 5.1 JSON File Storage Structure

All application data is stored in a single configuration file: `~/.vibemate/vibemate.json`

```
~/.vibemate/
└── vibemate.json         # Unified configuration file
```

> **Note**: Coding Agents are not stored in the configuration file but are auto-discovered from the system at runtime.

**Configuration File Structure**:

```
vibemate.json
├── app                   # Application configuration
│   ├── proxy_mode
│   ├── proxy_host
│   ├── proxy_port
│   ├── proxy_server_port
│   ├── theme
│   └── language
├── providers[]           # Provider list
│   ├── id
│   ├── name
│   ├── type
│   ├── api_base_url
│   ├── api_key
│   ├── is_default
│   ├── enable_proxy
│   └── status
└── routing_rules[]       # Routing rules list
    ├── id
    ├── provider_id  ──────► providers[].id
    ├── match_pattern
    ├── model_rewrite
    ├── priority
    └── enabled
```

### 5.2 Data Model Definitions

#### VibeMateConfig (Root Configuration Structure)

```rust
/// Unified configuration file structure (~/.vibemate/vibemate.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeMateConfig {
    pub app: AppConfig,
    pub providers: Vec<Provider>,
    pub routing_rules: Vec<RoutingRule>,
}

impl Default for VibeMateConfig {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            providers: Vec::new(),
            routing_rules: Vec::new(),
        }
    }
}
```

#### Provider

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,                  // UUID
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub api_base_url: String,
    pub api_key: String,             // Encrypted storage
    pub is_default: bool,
    pub enable_proxy: bool,
    pub status: ProviderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Google,
    Azure,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderStatus {
    Connected,
    Disconnected,
    Error,
}
```

#### RoutingRule

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub id: String,                  // UUID
    pub provider_id: String,         // References Provider.id
    pub match_pattern: String,       // Glob pattern, e.g., "gpt-4*"
    pub model_rewrite: Option<String>,
    pub priority: i32,               // Lower value = higher priority
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### CodingAgent (Auto-discovered from System)

Coding agents are not stored in the configuration file but are auto-discovered from the system at runtime.

```rust
/// Coding agent information (discovered at runtime, not persisted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingAgent {
    pub agent_type: AgentType,
    pub name: String,
    pub version: Option<String>,
    pub status: AgentStatus,
    pub executable_path: Option<String>,  // Executable file path
    pub config_path: Option<String>,      // Configuration file path
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentType {
    ClaudeCode,
    GeminiCLI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Installed,        // Installed
    NotInstalled,     // Not installed
    Authenticated,    // Authenticated
    NotAuthenticated, // Not authenticated
}

impl AgentType {
    /// Returns all supported agent types
    pub fn all() -> Vec<AgentType> {
        vec![AgentType::ClaudeCode, AgentType::GeminiCLI]
    }
    
    /// Get the display name for the agent
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "Claude Code",
            AgentType::GeminiCLI => "Gemini CLI",
        }
    }
    
    /// Get the command used to detect installation status
    pub fn detection_command(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "claude",
            AgentType::GeminiCLI => "gemini",
        }
    }
}
```

#### AppConfig (Application Configuration)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub proxy_mode: ProxyMode,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<u16>,
    pub proxy_server_port: u16,      // Local proxy server port, default 8080
    pub theme: Theme,
    pub language: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyMode {
    System,
    Custom,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            proxy_mode: ProxyMode::System,
            proxy_host: None,
            proxy_port: None,
            proxy_server_port: 8080,
            theme: Theme::Dark,
            language: "en".to_string(),
            updated_at: Utc::now(),
        }
    }
}
```

### 5.3 JSON Storage Implementation

```rust
// src-tauri/src/storage/config_store.rs

use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;
use crate::models::VibeMateConfig;

const CONFIG_FILE: &str = "vibemate.json";

pub struct ConfigStore {
    config_dir: PathBuf,
    config: RwLock<VibeMateConfig>,
}

impl ConfigStore {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            config: RwLock::new(VibeMateConfig::default()),
        }
    }

    /// Get configuration file path
    fn config_path(&self) -> PathBuf {
        self.config_dir.join(CONFIG_FILE)
    }

    /// Initialize storage (create directory and load configuration)
    pub async fn init(&self) -> Result<(), StorageError> {
        fs::create_dir_all(&self.config_dir).await?;
        self.load().await?;
        Ok(())
    }

    /// Load configuration from file
    pub async fn load(&self) -> Result<(), StorageError> {
        let path = self.config_path();
        let config = if path.exists() {
            let content = fs::read_to_string(&path).await?;
            serde_json::from_str(&content)?
        } else {
            VibeMateConfig::default()
        };
        *self.config.write().await = config;
        Ok(())
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<(), StorageError> {
        let path = self.config_path();
        let config = self.config.read().await;
        let content = serde_json::to_string_pretty(&*config)?;
        fs::write(&path, content).await?;
        Ok(())
    }

    /// Get complete configuration (read-only)
    pub async fn get_config(&self) -> VibeMateConfig {
        self.config.read().await.clone()
    }

    /// Update configuration and save
    pub async fn update<F>(&self, f: F) -> Result<(), StorageError>
    where
        F: FnOnce(&mut VibeMateConfig),
    {
        {
            let mut config = self.config.write().await;
            f(&mut config);
        }
        self.save().await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
```

#### Configuration File Example

**~/.vibemate/vibemate.json**
```json
{
  "app": {
    "proxy_mode": "System",
    "proxy_host": null,
    "proxy_port": null,
    "proxy_server_port": 8080,
    "theme": "Dark",
    "language": "en",
    "updated_at": "2026-01-10T10:00:00Z"
  },
  "providers": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "OpenAI",
      "type": "OpenAI",
      "api_base_url": "https://api.openai.com",
      "api_key": "encrypted:...",
      "is_default": true,
      "enable_proxy": true,
      "status": "Connected",
      "created_at": "2026-01-10T10:00:00Z",
      "updated_at": "2026-01-10T10:00:00Z"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440002",
      "name": "Anthropic",
      "type": "Anthropic",
      "api_base_url": "https://api.anthropic.com",
      "api_key": "encrypted:...",
      "is_default": false,
      "enable_proxy": true,
      "status": "Connected",
      "created_at": "2026-01-10T10:00:00Z",
      "updated_at": "2026-01-10T10:00:00Z"
    }
  ],
  "routing_rules": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "provider_id": "550e8400-e29b-41d4-a716-446655440000",
      "match_pattern": "gpt-4*",
      "model_rewrite": null,
      "priority": 1,
      "enabled": true,
      "created_at": "2026-01-10T10:00:00Z",
      "updated_at": "2026-01-10T10:00:00Z"
    },
    {
      "id": "550e8400-e29b-41d4-a716-446655440003",
      "provider_id": "550e8400-e29b-41d4-a716-446655440002",
      "match_pattern": "claude-*",
      "model_rewrite": null,
      "priority": 2,
      "enabled": true,
      "created_at": "2026-01-10T10:00:00Z",
      "updated_at": "2026-01-10T10:00:00Z"
    }
  ]
}
```

---

## 6. API Design

### 6.1 Tauri IPC Commands

#### Provider Commands

```typescript
// Frontend call interface
interface ProviderCommands {
  // Get all providers
  'provider:list': () => Promise<Provider[]>;
  
  // Create provider
  'provider:create': (input: CreateProviderInput) => Promise<Provider>;
  
  // Update provider
  'provider:update': (id: string, input: UpdateProviderInput) => Promise<Provider>;
  
  // Delete provider
  'provider:delete': (id: string) => Promise<void>;
  
  // Set default provider
  'provider:set_default': (id: string) => Promise<void>;
  
  // Test connection
  'provider:test_connection': (id: string) => Promise<ConnectionStatus>;
}
```

#### Router Commands

```typescript
interface RouterCommands {
  // Get all routing rules
  'router:list_rules': () => Promise<RoutingRule[]>;
  
  // Create routing rule
  'router:create_rule': (input: CreateRuleInput) => Promise<RoutingRule>;
  
  // Update routing rule
  'router:update_rule': (id: string, input: UpdateRuleInput) => Promise<RoutingRule>;
  
  // Delete routing rule
  'router:delete_rule': (id: string) => Promise<void>;
  
  // Reorder rules
  'router:reorder_rules': (ruleIds: string[]) => Promise<void>;
}
```

#### Agent Commands

```typescript
interface AgentCommands {
  // Discover all coding agents in the system (auto-detection)
  'agent:discover': () => Promise<CodingAgent[]>;
  
  // Check status of a specific agent
  'agent:check_status': (agentType: AgentType) => Promise<CodingAgent>;
  
  // Open login flow for an agent
  'agent:open_login': (agentType: AgentType) => Promise<void>;
  
  // Get agent version information
  'agent:get_version': (agentType: AgentType) => Promise<string | null>;
}
```

#### Network Commands

```typescript
interface NetworkCommands {
  // Get network configuration
  'network:get_config': () => Promise<NetworkConfig>;
  
  // Update network configuration
  'network:update_config': (config: UpdateNetworkConfig) => Promise<NetworkConfig>;
  
  // Test latency
  'network:test_latency': () => Promise<LatencyResult>;
}
```

#### System Commands

```typescript
interface SystemCommands {
  // Get proxy server status
  'system:proxy_status': () => Promise<ProxyStatus>;
  
  // Start proxy server
  'system:start_proxy': () => Promise<void>;
  
  // Stop proxy server
  'system:stop_proxy': () => Promise<void>;
  
  // Get application version
  'system:get_version': () => Promise<string>;
}
```

### 6.2 Rust Command Implementation Example

```rust
// src-tauri/src/commands/provider.rs

use tauri::State;
use std::sync::Arc;
use crate::services::ProviderService;
use crate::models::{Provider, CreateProviderInput, UpdateProviderInput};

#[tauri::command]
pub async fn list_providers(
    service: State<'_, Arc<ProviderService>>
) -> Result<Vec<Provider>, String> {
    service.list_providers()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_provider(
    service: State<'_, Arc<ProviderService>>,
    input: CreateProviderInput
) -> Result<Provider, String> {
    service.create_provider(input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_default_provider(
    service: State<'_, Arc<ProviderService>>,
    id: String
) -> Result<(), String> {
    service.set_default_provider(&id)
        .await
        .map_err(|e| e.to_string())
}
```

---

## 7. Frontend Component Design

### 7.1 Layout Components

```tsx
// src/components/layout/sidebar.tsx

interface SidebarItem {
  id: string;
  label: string;
  icon: LucideIcon;
  href: string;
}

const menuItems: SidebarItem[] = [
  { id: 'general', label: 'General', icon: Settings, href: '/general' },
  { id: 'providers', label: 'Model Provider', icon: Server, href: '/providers' },
  { id: 'router', label: 'Model Router', icon: GitMerge, href: '/router' },
  { id: 'agents', label: 'Coding Agents', icon: Bot, href: '/agents' },
  { id: 'network', label: 'Network', icon: Globe, href: '/network' },
];
```

### 7.2 Provider Card Component

```tsx
// src/components/providers/provider-card.tsx

interface ProviderCardProps {
  provider: Provider;
  onSetDefault: (id: string) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (id: string) => void;
}

export function ProviderCard({ provider, onSetDefault, onEdit, onDelete }: ProviderCardProps) {
  return (
    <Card className={cn(
      "transition-all duration-200 hover:shadow-lg hover:-translate-y-1",
      provider.isDefault && "ring-1 ring-primary/50"
    )}>
      <CardHeader>
        <div className="flex items-center gap-3">
          <ProviderLogo type={provider.type} />
          <CardTitle>{provider.name}</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2 text-sm font-mono">
          <div>API Base: {provider.apiBaseUrl}</div>
          <div>API Key: ••••••••••</div>
          <StatusBadge status={provider.status} />
        </div>
        <div className="flex items-center justify-between mt-4">
          <Label>Enable Proxy</Label>
          <Switch checked={provider.enableProxy} />
        </div>
      </CardContent>
      <CardFooter>
        <div className="flex items-center justify-between w-full">
          <Label>{provider.isDefault ? "System Default" : "Set as Default"}</Label>
          <Switch 
            checked={provider.isDefault} 
            onCheckedChange={() => onSetDefault(provider.id)}
          />
        </div>
      </CardFooter>
    </Card>
  );
}
```

### 7.3 Routing Rule Component

```tsx
// src/components/router/routing-rule-item.tsx

interface RoutingRuleItemProps {
  rule: RoutingRule;
  providers: Provider[];
  onUpdate: (rule: RoutingRule) => void;
  onDelete: (id: string) => void;
  dragHandleProps?: DraggableProvidedDragHandleProps;
}

export function RoutingRuleItem({ 
  rule, 
  providers, 
  onUpdate, 
  onDelete,
  dragHandleProps 
}: RoutingRuleItemProps) {
  return (
    <div className="flex items-center gap-4 p-4 bg-card rounded-lg border">
      {/* Drag Handle */}
      <div {...dragHandleProps} className="cursor-grab">
        <GripVertical className="w-5 h-5 text-muted-foreground" />
      </div>
      
      {/* Match Pattern */}
      <Input 
        className="font-mono w-40"
        placeholder="gpt-4*"
        value={rule.matchPattern}
        onChange={(e) => onUpdate({ ...rule, matchPattern: e.target.value })}
      />
      
      {/* Arrow */}
      <ArrowRight className="w-5 h-5 text-muted-foreground flex-shrink-0" />
      
      {/* Target Provider */}
      <Select 
        value={rule.providerId.toString()}
        onValueChange={(v) => onUpdate({ ...rule, providerId: parseInt(v) })}
      >
        <SelectTrigger className="w-40">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {providers.map(p => (
            <SelectItem key={p.id} value={p.id.toString()}>
              {p.name}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      
      {/* Model Rewrite */}
      <Input 
        className="font-mono w-48"
        placeholder="Leave empty to keep original"
        value={rule.modelRewrite || ''}
        onChange={(e) => onUpdate({ ...rule, modelRewrite: e.target.value || null })}
      />
      
      {/* Delete */}
      <Button variant="ghost" size="icon" onClick={() => onDelete(rule.id)}>
        <Trash2 className="w-4 h-4" />
      </Button>
    </div>
  );
}
```

### 7.4 State Management

```typescript
// src/stores/provider-store.ts

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface ProviderStore {
  providers: Provider[];
  isLoading: boolean;
  error: string | null;
  
  // Actions
  fetchProviders: () => Promise<void>;
  createProvider: (input: CreateProviderInput) => Promise<void>;
  updateProvider: (id: string, input: UpdateProviderInput) => Promise<void>;
  deleteProvider: (id: string) => Promise<void>;
  setDefaultProvider: (id: string) => Promise<void>;
}

export const useProviderStore = create<ProviderStore>((set, get) => ({
  providers: [],
  isLoading: false,
  error: null,
  
  fetchProviders: async () => {
    set({ isLoading: true, error: null });
    try {
      const providers = await invoke<Provider[]>('provider:list');
      set({ providers, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },
  
  setDefaultProvider: async (id: string) => {
    try {
      await invoke('provider:set_default', { id });
      // Update local state
      set(state => ({
        providers: state.providers.map(p => ({
          ...p,
          isDefault: p.id === id
        }))
      }));
    } catch (error) {
      set({ error: String(error) });
    }
  },
  
  // ... other actions
}));
```

---

## 8. Tauri Integration Design

### 8.1 Tauri Configuration

```json
// src-tauri/tauri.conf.json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Vibe Mate",
  "version": "0.1.0",
  "identifier": "com.vibe-mate.app",
  "build": {
    "frontendDist": "../out"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Vibe Mate",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "resizable": true,
        "decorations": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "plugins": {
    "shell": {
      "open": true
    }
  }
}
```

### 8.2 Main Program Entry

```rust
// src-tauri/src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod storage;
mod models;
mod services;
mod utils;

use storage::ConfigStore;
use services::{ProviderService, RouterService, AgentService, ConfigService, ProxyServer};
use std::sync::Arc;
use tauri::Manager;

/// Get config directory path (~/.vibemate/)
fn get_config_dir() -> std::path::PathBuf {
    let home = dirs::home_dir().expect("Failed to get home directory");
    home.join(".vibemate")
}

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Use ~/.vibemate/ as config directory
            let config_dir = get_config_dir();
            
            // Initialize unified config storage
            let store = Arc::new(ConfigStore::new(config_dir));
            tauri::async_runtime::block_on(async {
                store.init().await.expect("Failed to init storage");
            });
            
            // Initialize services
            let provider_service = Arc::new(ProviderService::new(store.clone()));
            let router_service = Arc::new(RouterService::new(store.clone(), provider_service.clone()));
            let agent_service = Arc::new(AgentService::new()); // AgentService automatically discovers agents in the system
            let config_service = Arc::new(ConfigService::new(store.clone()));
            let proxy_server = Arc::new(ProxyServer::new(router_service.clone()));

            // Register services to Tauri state management
            _app.manage(store);
            _app.manage(provider_service);
            _app.manage(router_service);
            _app.manage(agent_service);
            _app.manage(config_service);
            _app.manage(proxy_server.clone());

            // Automatically start proxy server when app launches
            tauri::async_runtime::spawn(async move {
                if let Err(e) = proxy_server.start(8080).await {
                    eprintln!("Failed to start proxy server: {}", e);
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Provider commands
            commands::provider::list_providers,
            commands::provider::create_provider,
            commands::provider::update_provider,
            commands::provider::delete_provider,
            commands::provider::set_default_provider,
            commands::provider::test_connection,
            // Router commands
            commands::router::list_rules,
            commands::router::create_rule,
            commands::router::update_rule,
            commands::router::delete_rule,
            commands::router::reorder_rules,
            // Agent commands
            commands::agent::discover_agents,
            commands::agent::check_status,
            commands::agent::open_login,
            commands::agent::get_version,
            // Config commands
            commands::config::get_config,
            commands::config::update_config,
            commands::network::test_latency,
            // System commands
            commands::system::proxy_status,
            commands::system::start_proxy,
            commands::system::stop_proxy,
            commands::system::get_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 9. Security Design

### 9.1 API Key Encrypted Storage

```rust
// src-tauri/src/utils/crypto.rs

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;

const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    pub fn new(key: &[u8; KEY_SIZE]) -> Self {
        Self {
            cipher: Aes256Gcm::new(key.into()),
        }
    }
    
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, CryptoError> {
        let nonce_bytes: [u8; NONCE_SIZE] = rand::thread_rng().gen();
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        // Return nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        Ok(result)
    }
    
    pub fn decrypt(&self, data: &[u8]) -> Result<String, CryptoError> {
        if data.len() < NONCE_SIZE {
            return Err(CryptoError::InvalidData);
        }
        
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        
        String::from_utf8(plaintext)
            .map_err(|_| CryptoError::InvalidUtf8)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid data")]
    InvalidData,
    #[error("Invalid UTF-8")]
    InvalidUtf8,
}
```

### 9.2 Key Management

```rust
// src-tauri/src/utils/keyring.rs

use keyring::Entry;

const SERVICE_NAME: &str = "vibe-mate";
const KEY_NAME: &str = "encryption-key";

pub struct KeyManager;

impl KeyManager {
    /// Get or generate encryption key
    pub fn get_or_create_key() -> Result<[u8; 32], KeyManagerError> {
        let entry = Entry::new(SERVICE_NAME, KEY_NAME)?;
        
        match entry.get_password() {
            Ok(key_hex) => {
                // Parse existing key
                hex::decode(&key_hex)
                    .map_err(|_| KeyManagerError::InvalidKey)?
                    .try_into()
                    .map_err(|_| KeyManagerError::InvalidKey)
            }
            Err(_) => {
                // Generate new key
                let key: [u8; 32] = rand::random();
                let key_hex = hex::encode(&key);
                entry.set_password(&key_hex)?;
                Ok(key)
            }
        }
    }
}
```

### 9.3 IPC Security

```rust
// Tauri command permission control
// src-tauri/capabilities/default.json

{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capability for Vibe Mate",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    {
      "identifier": "http:default",
      "allow": [
        { "url": "https://api.openai.com/*" },
        { "url": "https://api.anthropic.com/*" },
        { "url": "https://generativelanguage.googleapis.com/*" }
      ]
    }
  ]
}
```

---

## 10. Error Handling Design

### 10.1 Error Type Definitions

```rust
// src-tauri/src/utils/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    
    #[error("Router error: {0}")]
    Router(#[from] RouterError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("Provider not found: {0}")]
    NotFound(i64),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Invalid API key")]
    InvalidApiKey,
    
    #[error("Provider already exists: {0}")]
    AlreadyExists(String),
}

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Rule not found: {0}")]
    RuleNotFound(i64),
    
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    
    #[error("No matching provider for model: {0}")]
    NoMatch(String),
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection timeout")]
    Timeout,
    
    #[error("Proxy error: {0}")]
    ProxyError(String),
    
    #[error("Invalid configuration")]
    InvalidConfig,
}

// Implement Tauri serialization
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
```

### 10.2 Frontend Error Handling

```typescript
// src/lib/error.ts

export class AppError extends Error {
  constructor(
    message: string,
    public code: ErrorCode,
    public details?: unknown
  ) {
    super(message);
    this.name = 'AppError';
  }
}

export enum ErrorCode {
  NETWORK_ERROR = 'NETWORK_ERROR',
  PROVIDER_NOT_FOUND = 'PROVIDER_NOT_FOUND',
  INVALID_CONFIG = 'INVALID_CONFIG',
  UNAUTHORIZED = 'UNAUTHORIZED',
  UNKNOWN = 'UNKNOWN',
}

// Unified error handling Hook
export function useErrorHandler() {
  const { toast } = useToast();
  
  const handleError = useCallback((error: unknown) => {
    console.error('Error:', error);
    
    let message = 'An unexpected error occurred';
    
    if (error instanceof AppError) {
      message = error.message;
    } else if (error instanceof Error) {
      message = error.message;
    } else if (typeof error === 'string') {
      message = error;
    }
    
    toast({
      variant: 'destructive',
      title: 'Error',
      description: message,
    });
  }, [toast]);
  
  return { handleError };
}
```

---

## 11. UI/UX Design Specifications

### 11.1 Design Tokens

```typescript
// src/lib/constants.ts

export const DESIGN_TOKENS = {
  colors: {
    // Main background color
    background: '#09090b',
    // Card background
    card: '#18181b',
    // Border color
    border: '#27272a',
    // Primary color (Neon Purple)
    primary: '#a855f7',
    primaryHover: '#9333ea',
    // Accent color (Neon Blue)
    accent: '#3b82f6',
    accentHover: '#2563eb',
    // Status colors
    success: '#22c55e',
    warning: '#f59e0b',
    error: '#ef4444',
    // Text colors
    foreground: '#fafafa',
    mutedForeground: '#a1a1aa',
  },
  
  fonts: {
    sans: 'Inter, system-ui, sans-serif',
    mono: 'JetBrains Mono, Geist Mono, monospace',
  },
  
  radius: {
    sm: '0.25rem',
    md: '0.5rem',
    lg: '0.75rem',
    xl: '1rem',
  },
  
  spacing: {
    sidebar: '240px',
    contentMaxWidth: '1200px',
  },
} as const;
```

### 11.2 Tailwind Configuration

```typescript
// tailwind.config.ts

import type { Config } from 'tailwindcss';

const config: Config = {
  darkMode: 'class',
  content: ['./src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        card: {
          DEFAULT: 'hsl(var(--card))',
          foreground: 'hsl(var(--card-foreground))',
        },
        primary: {
          DEFAULT: 'hsl(var(--primary))',
          foreground: 'hsl(var(--primary-foreground))',
        },
        secondary: {
          DEFAULT: 'hsl(var(--secondary))',
          foreground: 'hsl(var(--secondary-foreground))',
        },
        muted: {
          DEFAULT: 'hsl(var(--muted))',
          foreground: 'hsl(var(--muted-foreground))',
        },
        accent: {
          DEFAULT: 'hsl(var(--accent))',
          foreground: 'hsl(var(--accent-foreground))',
        },
        destructive: {
          DEFAULT: 'hsl(var(--destructive))',
          foreground: 'hsl(var(--destructive-foreground))',
        },
        border: 'hsl(var(--border))',
        ring: 'hsl(var(--ring))',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Geist Mono', 'monospace'],
      },
      animation: {
        'glow': 'glow 2s ease-in-out infinite alternate',
        'slide-in': 'slideIn 0.2s ease-out',
        'fade-in': 'fadeIn 0.15s ease-out',
      },
      keyframes: {
        glow: {
          '0%': { boxShadow: '0 0 5px rgba(168, 85, 247, 0.5)' },
          '100%': { boxShadow: '0 0 20px rgba(168, 85, 247, 0.8)' },
        },
        slideIn: {
          '0%': { transform: 'translateX(-10px)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
      },
    },
  },
  plugins: [require('tailwindcss-animate')],
};

export default config;
```

### 11.3 Global Styles

```css
/* src/app/globals.css */

@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  :root {
    --background: 240 10% 3.9%;
    --foreground: 0 0% 98%;
    --card: 240 10% 7%;
    --card-foreground: 0 0% 98%;
    --popover: 240 10% 7%;
    --popover-foreground: 0 0% 98%;
    --primary: 270 91% 65%;
    --primary-foreground: 0 0% 98%;
    --secondary: 240 5% 17%;
    --secondary-foreground: 0 0% 98%;
    --muted: 240 5% 17%;
    --muted-foreground: 240 5% 65%;
    --accent: 217 91% 60%;
    --accent-foreground: 0 0% 98%;
    --destructive: 0 84% 60%;
    --destructive-foreground: 0 0% 98%;
    --border: 240 5% 17%;
    --input: 240 5% 17%;
    --ring: 270 91% 65%;
    --radius: 0.5rem;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  
  body {
    @apply bg-background text-foreground font-sans antialiased;
  }
  
  /* Custom scrollbar */
  ::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }
  
  ::-webkit-scrollbar-track {
    @apply bg-background;
  }
  
  ::-webkit-scrollbar-thumb {
    @apply bg-muted rounded-full;
  }
  
  ::-webkit-scrollbar-thumb:hover {
    @apply bg-muted-foreground/50;
  }
}

@layer components {
  /* Provider Card hover effect */
  .provider-card {
    @apply transition-all duration-200;
  }
  
  .provider-card:hover {
    @apply -translate-y-1 shadow-lg shadow-primary/10;
  }
  
  /* Default Provider glowing border */
  .provider-card-default {
    @apply ring-1 ring-primary/50;
    animation: glow 2s ease-in-out infinite alternate;
  }
  
  /* Routing rule drag styles */
  .routing-rule-dragging {
    @apply opacity-50 shadow-xl ring-2 ring-primary;
  }
  
  /* Status indicator */
  .status-dot {
    @apply w-2 h-2 rounded-full;
  }
  
  .status-dot-connected {
    @apply bg-green-500 animate-pulse;
  }
  
  .status-dot-disconnected {
    @apply bg-red-500;
  }
  
  .status-dot-error {
    @apply bg-yellow-500;
  }
}
```

### 11.4 Animation Design

```typescript
// src/lib/animations.ts

import { Variants } from 'motion/react';

// Page transition animation
export const pageVariants: Variants = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 },
};

// Card list animation
export const containerVariants: Variants = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

export const itemVariants: Variants = {
  hidden: { opacity: 0, y: 20 },
  show: { opacity: 1, y: 0 },
};

// Drag rule animation
export const dragVariants: Variants = {
  idle: { scale: 1, boxShadow: '0 0 0 rgba(0,0,0,0)' },
  dragging: { 
    scale: 1.02, 
    boxShadow: '0 10px 30px rgba(168, 85, 247, 0.3)',
    transition: { duration: 0.2 }
  },
};

// Sidebar menu item animation
export const menuItemVariants: Variants = {
  inactive: { 
    backgroundColor: 'transparent',
    x: 0,
  },
  active: { 
    backgroundColor: 'hsl(var(--accent) / 0.1)',
    x: 4,
    transition: { duration: 0.2 }
  },
};
```

---

## 12. Proxy Server Detailed Design

### 12.1 Proxy Server Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Proxy Server                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Listener   │───▶│   Router     │───▶│   Transformer    │  │
│  │  (Port 8080) │    │  (Matcher)   │    │   (Rewriter)     │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                    │             │
│                                                    ▼             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Response   │◀───│   Provider   │◀───│   Forwarder      │  │
│  │   Handler    │    │   Client     │    │                  │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 12.2 Proxy Server Implementation

```rust
// src-tauri/src/services/proxy.rs

use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProxyServer {
    router_service: Arc<RouterService>,
    state: Arc<RwLock<ProxyState>>,
}

#[derive(Default)]
struct ProxyState {
    is_running: bool,
    port: u16,
    request_count: u64,
}

impl ProxyServer {
    pub fn new(router_service: Arc<RouterService>) -> Self {
        Self {
            router_service,
            state: Arc::new(RwLock::new(ProxyState::default())),
        }
    }
    
    pub async fn start(&self, port: u16) -> Result<(), ProxyError> {
        let mut state = self.state.write().await;
        if state.is_running {
            return Err(ProxyError::AlreadyRunning);
        }
        
        let router_service = self.router_service.clone();
        let app = Router::new()
            .route("/v1/*path", axum::routing::any(proxy_handler))
            .with_state(router_service);
        
        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        
        state.is_running = true;
        state.port = port;
        
        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });
        
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<(), ProxyError> {
        let mut state = self.state.write().await;
        state.is_running = false;
        // Actual stop logic needs shutdown signal
        Ok(())
    }
    
    pub async fn status(&self) -> ProxyStatus {
        let state = self.state.read().await;
        ProxyStatus {
            is_running: state.is_running,
            port: state.port,
            request_count: state.request_count,
        }
    }
}

async fn proxy_handler(
    State(router_service): State<Arc<RouterService>>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    // 1. Parse model parameter from request
    let model_name = extract_model_from_request(&req).await?;
    
    // 2. Match provider using router service
    let resolved = router_service
        .match_provider(&model_name)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    // 3. Transform request (rewrite model, update headers)
    let transformed_req = transform_request(req, &resolved).await?;
    
    // 4. Forward to target provider
    let client = reqwest::Client::new();
    let response = client
        .request(transformed_req.method().clone(), &resolved.api_url)
        .headers(transformed_req.headers().clone())
        .body(reqwest::Body::from(
            axum::body::to_bytes(transformed_req.into_body(), usize::MAX)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        ))
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    // 5. Return response
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    let mut res = Response::builder().status(status);
    *res.headers_mut().unwrap() = headers;
    res.body(Body::from(body)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Debug, Clone)]
pub struct ResolvedProvider {
    pub provider: Provider,
    pub api_url: String,
    pub model_name: String,  // Model name (may be rewritten)
}

async fn extract_model_from_request(req: &Request<Body>) -> Result<String, StatusCode> {
    // Extract model field from request body
    // OpenAI format: { "model": "gpt-4", ... }
    todo!()
}

async fn transform_request(
    req: Request<Body>,
    resolved: &ResolvedProvider,
) -> Result<Request<Body>, StatusCode> {
    // 1. Replace API Key header
    // 2. Replace model field (if rewrite exists)
    // 3. Update Host header
    todo!()
}
```

### 12.3 Model Routing Match Algorithm

```rust
// src-tauri/src/services/router.rs

use glob::Pattern;

impl RouterService {
    /// Match routing rules by model name
    pub async fn match_provider(&self, model_name: &str) -> Result<ResolvedProvider, RouterError> {
        // 1. Get all enabled rules (sorted by priority)
        let rules = self.list_rules().await?;
        let enabled_rules: Vec<_> = rules
            .into_iter()
            .filter(|r| r.enabled)
            .collect();
        
        // 2. Match in priority order
        for rule in &enabled_rules {
            if self.matches_pattern(&rule.match_pattern, model_name)? {
                let provider = self.provider_service
                    .get_provider(&rule.provider_id)
                    .await?;
                
                return Ok(ResolvedProvider {
                    provider: provider.clone(),
                    api_url: format!("{}/v1/chat/completions", provider.api_base_url),
                    model_name: rule.model_rewrite
                        .as_ref()
                        .unwrap_or(&model_name.to_string())
                        .clone(),
                });
            }
        }
        
        // 3. Fallback to default provider
        let default_provider = self.provider_service
            .get_default_provider()
            .await?
            .ok_or(RouterError::NoDefaultProvider)?;
        
        Ok(ResolvedProvider {
            provider: default_provider.clone(),
            api_url: format!("{}/v1/chat/completions", default_provider.api_base_url),
            model_name: model_name.to_string(),
        })
    }
    
    /// Glob pattern matching
    fn matches_pattern(&self, pattern: &str, model_name: &str) -> Result<bool, RouterError> {
        let pattern = Pattern::new(pattern)
            .map_err(|_| RouterError::InvalidPattern(pattern.to_string()))?;
        Ok(pattern.matches(model_name))
    }
}
```

---

## 13. Testing Strategy

### 13.1 Backend Tests

```rust
// src-tauri/src/services/router_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pattern_matching() {
        let service = RouterService::new_for_test().await;
        
        // Exact match
        assert!(service.matches_pattern("gpt-4", "gpt-4").unwrap());
        assert!(!service.matches_pattern("gpt-4", "gpt-4-turbo").unwrap());
        
        // Wildcard match
        assert!(service.matches_pattern("gpt-4*", "gpt-4").unwrap());
        assert!(service.matches_pattern("gpt-4*", "gpt-4-turbo").unwrap());
        assert!(service.matches_pattern("claude-*", "claude-3-5-sonnet").unwrap());
        
        // Complex patterns
        assert!(service.matches_pattern("*-turbo", "gpt-4-turbo").unwrap());
        assert!(service.matches_pattern("claude-3-?-*", "claude-3-5-sonnet").unwrap());
    }
    
    #[tokio::test]
    async fn test_rule_priority() {
        let service = RouterService::new_for_test().await;
        
        // Create rule: priority 1 matches gpt-4* -> Provider A
        // Create rule: priority 2 matches gpt-* -> Provider B
        
        let resolved = service.match_provider("gpt-4-turbo").await.unwrap();
        assert_eq!(resolved.provider.name, "Provider A");
        
        let resolved = service.match_provider("gpt-3.5-turbo").await.unwrap();
        assert_eq!(resolved.provider.name, "Provider B");
    }
}
```

### 13.2 Frontend Tests

```typescript
// src/components/providers/__tests__/provider-card.test.tsx

import { render, screen, fireEvent } from '@testing-library/react';
import { ProviderCard } from '../provider-card';

describe('ProviderCard', () => {
  const mockProvider = {
    id: 1,
    name: 'OpenAI',
    type: 'OpenAI',
    apiBaseUrl: 'https://api.openai.com',
    isDefault: false,
    enableProxy: true,
    status: 'Connected',
  };

  it('renders provider information', () => {
    render(<ProviderCard provider={mockProvider} />);
    
    expect(screen.getByText('OpenAI')).toBeInTheDocument();
    expect(screen.getByText('https://api.openai.com')).toBeInTheDocument();
  });

  it('shows default badge when isDefault is true', () => {
    render(<ProviderCard provider={{ ...mockProvider, isDefault: true }} />);
    
    expect(screen.getByText('System Default')).toBeInTheDocument();
  });

  it('calls onSetDefault when toggle is clicked', () => {
    const onSetDefault = jest.fn();
    render(<ProviderCard provider={mockProvider} onSetDefault={onSetDefault} />);
    
    fireEvent.click(screen.getByRole('switch'));
    expect(onSetDefault).toHaveBeenCalledWith(1);
  });
});
```

### 13.3 E2E Tests

```typescript
// e2e/provider.spec.ts

import { test, expect } from '@playwright/test';

test.describe('Provider Management', () => {
  test('should add a new provider', async ({ page }) => {
    await page.goto('/providers');
    
    // Click add button
    await page.click('button:has-text("Add Provider")');
    
    // Fill form
    await page.fill('input[name="name"]', 'Test Provider');
    await page.fill('input[name="apiBaseUrl"]', 'https://api.test.com');
    await page.fill('input[name="apiKey"]', 'test-key-12345');
    
    // Submit
    await page.click('button:has-text("Save")');
    
    // Verify
    await expect(page.locator('text=Test Provider')).toBeVisible();
  });

  test('should set default provider', async ({ page }) => {
    await page.goto('/providers');
    
    // Find non-default provider card
    const card = page.locator('.provider-card').filter({ hasNot: page.locator('.provider-card-default') }).first();
    
    // Click Set as Default switch
    await card.locator('text=Set as Default').click();
    
    // Verify the card now shows System Default
    await expect(card.locator('text=System Default')).toBeVisible();
  });
});
```

---

## 14. Deployment and Release

### 14.1 Build Configuration

```json
// package.json (partial)
{
  "scripts": {
    "dev": "next dev",
    "build": "next build && next export",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build",
    "lint": "eslint . --ext .ts,.tsx",
    "test": "jest",
    "test:e2e": "playwright test"
  }
}
```

### 14.2 GitHub Actions CI/CD

```yaml
# .github/workflows/release.yml

name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu

    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '22'
          cache: 'pnpm'
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install dependencies
        run: pnpm install
      
      - name: Build frontend
        run: pnpm build
      
      - name: Build Tauri
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: 'Vibe Mate v__VERSION__'
          releaseBody: 'See the assets to download this version and install.'
          releaseDraft: true
          prerelease: false
```

### 14.3 Auto Update Configuration

```rust
// src-tauri/src/updater.rs

use tauri::updater::UpdaterBuilder;

pub fn setup_updater(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();
    
    tauri::async_runtime::spawn(async move {
        match handle.updater_builder().build() {
            Ok(updater) => {
                if let Ok(Some(update)) = updater.check().await {
                    // Notify frontend that new version is available
                    handle.emit_all("update-available", &update.version).ok();
                }
            }
            Err(e) => eprintln!("Updater error: {}", e),
        }
    });
    
    Ok(())
}
```

```json
// src-tauri/tauri.conf.json (add updater config)
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.vibe-mate.app/{{target}}/{{arch}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

---

## 15. Appendix

### 15.1 Development Environment Requirements

| Tool | Minimum Version | Description |
|------|-----------------|-------------|
| Node.js | v22.0.0 | JavaScript runtime (LTS) |
| pnpm | v9.0.0 | Package manager |
| Rust | v1.84.0 | System programming language |
| Cargo | v1.84.0 | Rust package manager |
| Tauri CLI | v2.2.0 | Tauri command line tool |

#### macOS Additional Dependencies

```bash
# Xcode Command Line Tools
xcode-select --install

# Install dependencies via Homebrew
brew install pkg-config
```

#### Windows Additional Dependencies

```powershell
# Visual Studio Build Tools
# Download and install Visual Studio Build Tools 2022
# Select "Desktop development with C++"

# WebView2 (usually pre-installed on Windows 10/11)
# https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

#### Linux Additional Dependencies

```bash
# Debian/Ubuntu
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

### 15.2 Project Initialization Steps

```bash
# 1. Clone repository
git clone https://github.com/your-org/vibe-mate.git
cd vibe-mate

# 2. Install frontend dependencies
pnpm install

# 3. Install Rust dependencies
cd src-tauri
cargo build

# 4. Start development server
cd ..
pnpm tauri:dev
```

### 15.3 Common Commands

| Command | Description |
|---------|-------------|
| `pnpm dev` | Start Next.js dev server |
| `pnpm build` | Build frontend production version |
| `pnpm tauri:dev` | Start Tauri dev mode |
| `pnpm tauri:build` | Build desktop app |
| `pnpm lint` | Run ESLint check |
| `pnpm test` | Run unit tests |
| `pnpm test:e2e` | Run E2E tests |
| `cargo test` | Run Rust tests |
| `cargo clippy` | Rust code linting |
| `cargo fmt` | Rust code formatting |

### 15.4 Environment Variable Configuration

```bash
# .env.local (development environment)

# Frontend configuration
NEXT_PUBLIC_APP_NAME=Vibe Mate
NEXT_PUBLIC_VERSION=$npm_package_version

# Tauri configuration (configured in tauri.conf.json)
```

```rust
// Backend environment variables (via Tauri config or runtime)

// Proxy server port (default 8080)
// PROXY_PORT=8080

// Log level
// RUST_LOG=info
```

**Config File Location**:
- Unified across all platforms: `~/.vibemate/vibemate.json`
  - macOS: `/Users/<username>/.vibemate/vibemate.json`
  - Windows: `C:\Users\<username>\.vibemate\vibemate.json`
  - Linux: `/home/<username>/.vibemate/vibemate.json`

### 15.5 Directory Structure

```
vibe-mate/
├── .github/                    # GitHub configuration
│   └── workflows/              # CI/CD workflows
├── docs/                       # Project documentation
│   ├── prd.md                  # Product requirements document
│   └── system_design.md        # System design document
├── public/                     # Static assets
│   └── icons/                  # Application icons
├── src/                        # Next.js frontend source
│   ├── app/                    # App Router pages
│   ├── components/             # React components
│   ├── hooks/                  # Custom Hooks
│   ├── lib/                    # Utility library
│   ├── stores/                 # Zustand state
│   └── types/                  # TypeScript types
├── src-tauri/                  # Tauri backend source
│   ├── src/                    # Rust source
│   │   ├── storage/            # Config storage
│   │   ├── services/           # Business services
│   │   ├── models/             # Data models
│   │   ├── commands/           # Tauri IPC commands
│   │   └── utils/              # Utility functions
│   ├── capabilities/           # Permission configuration
│   ├── icons/                  # Application icons
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
├── e2e/                        # E2E tests
├── package.json                # Node.js dependencies
├── tailwind.config.ts          # Tailwind configuration
├── tsconfig.json               # TypeScript configuration
└── next.config.js              # Next.js configuration
```

### 15.6 References

| Resource | Link |
|----------|------|
| Tauri Docs | https://v2.tauri.app/start/ |
| Next.js Docs | https://nextjs.org/docs |
| React Docs | https://react.dev/ |
| Shadcn/UI Docs | https://ui.shadcn.com/ |
| Tailwind CSS Docs | https://tailwindcss.com/docs |
| Axum Docs | https://docs.rs/axum/latest/axum/ |
| Zustand Docs | https://docs.pmnd.rs/zustand/getting-started/introduction |
| TanStack Query Docs | https://tanstack.com/query/latest |
| Motion Docs | https://motion.dev/ |

### 15.7 Glossary

| Term | Definition |
|------|------------|
| Provider | AI model provider, such as OpenAI, Anthropic, Google, etc. |
| Routing Rule | Model routing rule that defines how requests are distributed to different providers |
| Coding Agent | Coding assistant like Claude Code, Gemini CLI, etc., automatically discovered by the system |
| Proxy Server | Proxy server that intercepts and forwards AI API requests |
| IPC | Inter-Process Communication |
| Glob Pattern | Wildcard pattern used to match model names |
| Model Rewrite | Model rewrite, replacing the model name in request with target model |
| Catch-All | Fallback rule, uses default provider when no rules match |

---

## 16. Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| v0.1.0 | 2026-01-10 | - | Initial version, complete basic architecture design |

---

## 17. TODO

- [ ] Improve proxy server streaming response support (SSE)
- [ ] Add request logging and statistics
- [ ] Implement internationalization support (i18n)
- [ ] Add system tray functionality
- [ ] Implement configuration import/export
- [ ] Add API call statistics and cost estimation
- [ ] Implement provider health check and automatic failover