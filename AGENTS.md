# Repository Guidelines

## Project Structure & Module Organization
- `src/` holds the React app (`App.tsx`/`main.tsx` entry). `components/` contains reusable UI, `pages/` route screens, `hooks/` side-effects and Tauri bridges, `stores/` Zustand state, `lib/` shared utilities, `types/` shared typings, `assets/` static assets, and `index.css` defines the Tailwind theme tokens.
- `src-tauri/` contains the Rust backend (commands, HTTP proxy, plugins) with `tauri.conf.json` for window/build settings, `capabilities/` for permissions, and bundling assets under `icons/`; build artifacts land in `src-tauri/target/`.
- `public/` serves static files at dev time; `dist/` is the Vite build output consumed by Tauri; `docs/` houses design/system notes when present.

## Build, Test, and Development Commands
- `pnpm dev` — Vite dev server for the web frontend at http://localhost:5173.
- `pnpm tauri dev` — Full desktop run; executes `pnpm dev` then launches the Tauri window.
- `pnpm build` — Type-checks via `tsc` then produces optimized assets in `dist/`.
- `pnpm preview` — Serves the built site locally for QA before packaging.
- `pnpm tauri build` — Bundles the desktop app (Rust toolchain and `cargo` required).

## Coding Style & Naming Conventions
- TypeScript in strict mode; React 19 with React Router and TanStack Query. Use the `@/*` alias for intra-app imports.
- Components/pages use PascalCase filenames (`DashboardPage.tsx`, `Sidebar.tsx`); hooks live in `src/hooks/` and start with `use*`; Zustand stores in `src/stores/`.
- Styling uses Tailwind v4 utilities with design tokens in `src/index.css`; prefer utility classes plus `clsx`/`cva` for variants.
- Keep side-effects in hooks, cache network calls with React Query, and isolate reusable logic in `lib/`.

## Testing Guidelines
- No automated test runner is wired yet; add Vitest + React Testing Library when introducing tests.
- Co-locate tests as `*.test.ts`/`*.test.tsx` near the code they cover; include integration coverage for routes/components and unit coverage for utilities.
- Manually validate desktop flows after changes touching Tauri commands, proxy logic, or capabilities.

## Commit & Pull Request Guidelines
- Follow conventional commits (`feat:`, `fix:`, `docs:`) as seen in history.
- PRs should describe scope, risks, and manual validation steps; link related issues. Include screenshots/GIFs for UI updates and note Tauri capability or plugin changes.
- Keep diffs small and focused; update docs/config when behavior changes.
