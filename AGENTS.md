# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build, Test, and Development Commands

### Frontend Development
- `pnpm dev` — Vite dev server for web frontend at http://localhost:1420
- `pnpm build` — Type-checks via `tsc` then produces optimized assets in `dist/`
- `pnpm preview` — Serves the built site locally for QA before packaging

### Full Desktop Development
- `pnpm tauri dev` — Full desktop run; executes `pnpm dev` then launches the Tauri window
- `pnpm tauri build` — Bundles the desktop app (Rust toolchain and `cargo` required)
- `pnpm tauri build --debug` — Development build with debug symbols

### Rust Backend
- `cd src-tauri && cargo build` — Build Rust backend only
- `cd src-tauri && cargo test` — Run Rust tests
- `cd src-tauri && cargo clippy` — Lint Rust code

## Project Structure & Module Organization

### Frontend (`src/`)
- `main.tsx` — Application bootstrap
- `App.tsx` — Root component with routing setup
- `index.css` — Tailwind v4 theme tokens and design system
- `components/` — Reusable UI components
  - `layout/` — Layout components (Sidebar, MainContent)
  - `ui/` — shadcn/ui components (Button, Dialog, etc.)
- `pages/` — Route screens (Dashboard, Providers, Agents, Routing, Settings)
- `hooks/` — Custom React hooks and Tauri IPC bridges
- `stores/` — Zustand state management stores
- `lib/` — Shared utilities and helper functions
- `types/` — TypeScript type definitions
- `assets/` — App-imported static assets

### Backend (`src-tauri/`)
- `src/main.rs` — Entry point that calls `vibe_mate_lib::run()`
- `src/lib.rs` — Core initialization: tracing setup, ConfigStore init, service registration, proxy auto-start
- `src/commands/` — Tauri IPC handlers (22 commands total):
  - `provider.rs` — 6 provider management commands
  - `router.rs` — 5 routing rule commands
  - `agent.rs` — 6 agent discovery/management commands
  - `config.rs` — 5 configuration commands
  - `system.rs` — 4 system/proxy commands
- `src/services/` — Business logic layer:
  - `provider.rs` — Provider CRUD and default management
  - `router.rs` — Routing rule management with glob pattern matching
  - `agent.rs` — Agent discovery, version detection, platform-specific login flows
  - `config.rs` — App configuration management
  - `proxy.rs` — HTTP proxy server (900+ lines) - intelligent API request routing
- `src/models/` — Data structures (Provider, RoutingRule, CodingAgent, AppConfig)
- `src/storage/` — Persistent storage layer
  - `config_store.rs` — Unified JSON-based storage at `~/.vibemate/settings.json`
- `capabilities/` — Tauri permission definitions
- `tauri.conf.json` — Window and build settings
- `icons/` — Application icons for all platforms
- `target/` — Rust build output

### Configuration Files
- `vite.config.ts` — Vite config with `@/*` alias, Tauri-specific settings, port 1420
- `tsconfig.json` / `tsconfig.node.json` — TypeScript configuration
- `package.json` — Dependencies and scripts
- `Cargo.toml` — Rust dependencies and build settings
- `public/` — Static files served at dev time
- `dist/` — Vite build output consumed by Tauri
- `docs/` — Design and system documentation

## Architecture Overview

### Proxy System (Core Feature)
The application runs an HTTP proxy server on port 12345 that intelligently routes LLM API requests:

**Request Flow**:
1. Agent (Claude Code/Codex/Gemini) sends API request to proxy
2. Proxy extracts model from JSON body
3. Proxy looks up routing rules (by priority, filtered by API group)
4. Proxy resolves target provider (via rules or default)
5. Proxy optionally rewrites model field for provider compatibility
6. Proxy forwards request to provider's API endpoint with auth headers
7. Proxy handles streaming (SSE) or regular responses

**Routing Logic** (`src-tauri/src/services/proxy.rs`):
- Three handlers: `openai_proxy_handler`, `anthropic_proxy_handler`, `generic_proxy_handler`
- Pattern matching using glob patterns (e.g., `gpt-4*`, `claude-*`)
- Priority-based rule evaluation
- Model rewriting for cross-provider compatibility
- Respects global proxy settings with no_proxy list support

**Provider Resolution**:
- Model-based rules checked first (exact model matching)
- Path-based rules as fallback (URL pattern matching)
- Default provider if no rules match
- All rules must be enabled

### Storage Architecture
- **Single source of truth**: `~/.vibemate/settings.json`
- **ConfigStore**: Arc<RwLock<VibeMateConfig>> for thread-safe access
- **Atomic updates**: Closure-based `store.update(|config| { ... })` pattern
- **Lazy loading**: Services load config on-demand via ConfigStore

### Service Layer Pattern
- Services injected via Tauri state management (Arc-wrapped)
- Commands extract services from state: `state.inner().clone()`
- Services interact with ConfigStore for persistence
- All async operations use Tokio runtime

### Agent Management
- Platform-specific login flows:
  - macOS: osascript Terminal integration
  - Linux: gnome-terminal → konsole → xterm fallback
  - Windows: cmd.exe integration
- Default config paths:
  - Claude Code: `~/.claude/settings.json`
  - Codex: `~/.codex/config.toml`
  - Gemini CLI: `~/.gemini/settings.json`
- Tilde path expansion supported

## Coding Style & Naming Conventions

### TypeScript/React
- Strict mode enabled; React 19 with React Router 7 and TanStack Query
- Use `@/*` alias for all intra-app imports
- Components/pages: PascalCase filenames (`DashboardPage.tsx`, `Sidebar.tsx`)
- Hooks: Prefix with `use*`, place in `src/hooks/`
- Stores: Zustand stores in `src/stores/`
- Styling: Tailwind v4 utilities + `clsx`/`cva` for variants
- Keep side-effects in hooks, cache network calls with React Query
- Isolate reusable logic in `lib/`

### Rust
- Async/await throughout with Tokio runtime
- Error handling: Custom types with `thiserror`, `anyhow` for flexibility
- Shared state: Arc<RwLock<T>> or Arc<Service>
- Commands: Return `Result<T, String>` for Tauri IPC
- Logging: Use `tracing` macros (debug!, info!, warn!, error!)
- Serialization: Derive `Serialize`, `Deserialize` for all models

## Testing Guidelines
- No automated test runner wired yet; add Vitest + React Testing Library when introducing tests
- Co-locate tests as `*.test.ts`/`*.test.tsx` near the code they cover
- Include integration coverage for routes/components and unit coverage for utilities
- Manually validate desktop flows after changes to:
  - Tauri commands or capabilities
  - Proxy routing logic
  - Agent detection/login flows
  - Cross-platform behavior

## Commit & Pull Request Guidelines
- Follow conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`
- Keep diffs small and focused
- PRs should describe scope, risks, and manual validation steps
- Link related issues in PR description
- Include screenshots/GIFs for UI updates
- Note any Tauri capability or plugin changes
- Update docs/config when behavior changes
