# Repository Guidelines

## Project Structure & Module Organization
Core layout (abridged):
```text
.
├─ src/                 # React frontend
│  ├─ components/       # UI and feature components
│  ├─ pages/            # Route screens
│  ├─ hooks/            # Custom hooks and IPC bridges
│  ├─ stores/           # Zustand stores
│  ├─ lib/              # Shared utilities
│  ├─ types/            # TypeScript types
│  └─ assets/           # App-imported assets
├─ src-tauri/           # Rust backend (Tauri)
│  └─ src/
│     ├─ commands/      # Tauri IPC handlers
│     ├─ services/      # Business logic
│     ├─ models/        # Data structures
│     ├─ storage/       # Persistent config store
│     └─ agents/        # Agent discovery + integration
├─ public/              # Static assets (dev-time)
├─ docs/                # Design/system documentation
└─ dist/                # Vite build output (consumed by Tauri)
```

## Build, Test, and Development Commands
- `pnpm dev` runs the Vite dev server at `http://localhost:1420`.
- `pnpm build` type-checks with `tsc` and builds production assets.
- `pnpm preview` serves the built frontend for QA.
- `pnpm tauri dev` launches the full desktop app (Vite + Tauri).
- `pnpm tauri build` or `pnpm tauri build --debug` bundles the desktop app.
- `cd src-tauri && cargo build/test/clippy` builds, tests, or lints the Rust backend.

## Coding Style & Naming Conventions
- TypeScript/React: strict mode, React 19, React Router 7, Zustand, Tailwind v4.
- Use the `@/*` alias for internal imports.
- Components and pages use PascalCase filenames (e.g., `DashboardPage.tsx`). Hooks are `use*` in `src/hooks/`. Stores live in `src/stores/`.
- Tailwind utility styling with `clsx` and `class-variance-authority` for variants.
- Rust is async-first with Tokio; errors use `thiserror`/`anyhow`, logging via `tracing`.
- No dedicated formatter/linter scripts are defined in `package.json`; match existing style and keep diffs focused.

## Testing Guidelines
- No frontend test runner is wired yet; place tests near code as `*.test.ts`/`*.test.tsx` when added.
- Rust tests run via `cd src-tauri && cargo test`.
- Manually validate changes that touch Tauri commands, proxy routing, or agent discovery/login flows.

## Commit & Pull Request Guidelines
- Use conventional commits seen in history: `feat:`, `fix:`, `refactor:`, `chore:`, `docs:`.
- PRs should explain scope, risks, and manual validation steps. Include screenshots/GIFs for UI changes.
- Link related issues and note any Tauri capability or configuration changes.

## Architecture & Configuration Notes
- The app runs a local HTTP proxy (port `12345`) for routing LLM API requests by model and rule priority.
- Persistent settings live at `~/.vibemate/settings.json` via the ConfigStore.
