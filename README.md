<div align="center">
  <img src="docs/images/banner.png" alt="Vibe Mate Banner" width="800"/>

  <h1>Vibe Mate</h1>

  <p><strong>A modern desktop application for managing AI model providers, agents, and routing rules</strong></p>

  [![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/zhenlohuang/vibe-mate)
  [![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
  [![Tauri](https://img.shields.io/badge/Tauri-2.0-orange.svg)](https://tauri.app)
  [![React](https://img.shields.io/badge/React-19-blue.svg)](https://react.dev)

  [Features](#features) • [Tech Stack](#tech-stack) • [Getting Started](#getting-started) • [Development](#development) • [Contributing](#contributing)
</div>

---

## ✨ Features

- **🤖 AI Agent Management** - Create, configure, and manage multiple AI agents with customizable settings
- **🔌 Provider Management** - Connect and manage multiple AI model providers (OpenAI, Anthropic, etc.)
- **🔀 Smart Routing** - Configure intelligent routing rules to distribute requests across providers
- **📊 Dashboard** - Real-time monitoring of proxy status and system metrics
- **⚙️ Settings Management** - Centralized configuration for network, proxy, and application settings
- **🎨 Modern UI** - Beautiful, responsive interface built with Radix UI and Tailwind CSS
- **🔒 Desktop Native** - Cross-platform desktop app with native performance powered by Tauri

## 🛠 Tech Stack

### Frontend
- **[React 19](https://react.dev)** - Modern UI library with latest features
- **[TypeScript](https://www.typescriptlang.org/)** - Type-safe development
- **[Vite](https://vitejs.dev/)** - Lightning-fast build tool
- **[React Router](https://reactrouter.com/)** - Client-side routing
- **[TanStack Query](https://tanstack.com/query)** - Powerful data synchronization
- **[Zustand](https://zustand-demo.pmnd.rs/)** - Lightweight state management
- **[Tailwind CSS v4](https://tailwindcss.com/)** - Utility-first styling
- **[Radix UI](https://www.radix-ui.com/)** - Accessible component primitives
- **[Motion](https://motion.dev/)** - Smooth animations
- **[Lucide Icons](https://lucide.dev/)** - Beautiful icon library

### Backend
- **[Tauri](https://tauri.app/)** - Secure desktop app framework
- **[Rust](https://www.rust-lang.org/)** - High-performance backend

### Package Management
- **[pnpm](https://pnpm.io/)** - Fast, disk space efficient package manager

## 📁 Project Structure

```
vibe-mate/
├── src/                      # React frontend source
│   ├── components/          # Reusable UI components
│   │   ├── layout/         # Layout components (Sidebar, MainContent)
│   │   └── ui/             # shadcn/ui components
│   ├── pages/              # Route pages (Dashboard, Providers, Agents, etc.)
│   ├── hooks/              # Custom React hooks and Tauri bridges
│   ├── stores/             # Zustand state stores
│   ├── lib/                # Shared utilities
│   ├── types/              # TypeScript type definitions
│   ├── assets/             # Static assets
│   ├── App.tsx             # Root component
│   ├── main.tsx            # Application entry point
│   └── index.css           # Tailwind theme tokens
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── commands/       # Tauri commands (agent, provider, router, etc.)
│   │   ├── models/         # Data models
│   │   ├── services/       # Business logic
│   │   ├── storage/        # Persistent storage
│   │   ├── main.rs         # Entry point
│   │   └── lib.rs          # Shared module
│   ├── capabilities/       # Tauri permissions
│   ├── icons/              # Application icons
│   └── tauri.conf.json     # Tauri configuration
├── public/                  # Static files
├── dist/                    # Vite build output
├── docs/                    # Documentation and design notes
├── AGENTS.md               # Developer guidelines (same as CLAUDE.md)
└── README.md               # This file
```

## 🚀 Getting Started

### Prerequisites

- **Node.js** v18+ ([Download](https://nodejs.org/))
- **pnpm** v10+ ([Install](https://pnpm.io/installation))
- **Rust** ([Install](https://www.rust-lang.org/tools/install))
- **Tauri CLI** (installed via pnpm)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/vibe-mate.git
   cd vibe-mate
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Run the development app**
   ```bash
   pnpm tauri dev
   ```

The application will launch in a native desktop window.

### Alternative: Web-only Development

To run just the frontend in a browser for rapid UI development:

```bash
pnpm dev
```

Visit http://localhost:1420 in your browser.

## 💻 Development

### Available Commands

| Command | Description |
|---------|-------------|
| `pnpm dev` | Start Vite dev server for web frontend |
| `pnpm tauri dev` | Launch full desktop app with hot reload |
| `pnpm build` | Type-check and build optimized assets |
| `pnpm preview` | Preview production build locally |
| `pnpm tauri build` | Build production desktop application |

### Coding Guidelines

- **TypeScript**: Strict mode enabled, use type-safe patterns
- **Components**: PascalCase filenames (`DashboardPage.tsx`, `Sidebar.tsx`)
- **Hooks**: Prefix with `use*` and place in `src/hooks/`
- **Stores**: Zustand stores in `src/stores/`
- **Styling**: Tailwind utilities + `clsx`/`cva` for variants
- **Imports**: Use `@/*` alias for intra-app imports
- **Commits**: Follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`)

See [AGENTS.md](AGENTS.md) for detailed development guidelines.

## 🏗 Building

### Development Build

```bash
pnpm tauri build --debug
```

### Production Build

```bash
pnpm tauri build
```

Build artifacts will be in `src-tauri/target/release/bundle/`.

### Platform-specific Builds

```bash
# macOS (.dmg, .app)
pnpm tauri build --target universal-apple-darwin

# Windows (.exe, .msi)
pnpm tauri build --target x86_64-pc-windows-msvc

# Linux (.deb, .appimage)
pnpm tauri build --target x86_64-unknown-linux-gnu
```

## 🧪 Testing

Currently manual testing is in place. To add automated tests:

1. Install testing dependencies:
   ```bash
   pnpm add -D vitest @testing-library/react @testing-library/jest-dom
   ```

2. Add test scripts to `package.json`

3. Co-locate tests as `*.test.ts`/`*.test.tsx` near source files

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

1. **Fork the repository** and create a feature branch
2. **Follow coding standards** outlined in [AGENTS.md](AGENTS.md)
3. **Write clear commit messages** using Conventional Commits
4. **Test thoroughly** - validate desktop flows and UI changes
5. **Submit a PR** with detailed description, screenshots for UI changes

### Pull Request Guidelines

- Keep diffs small and focused
- Include manual validation steps
- Note any Tauri capability or permission changes
- Link related issues
- Add screenshots/GIFs for UI updates

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/)
- UI components from [Radix UI](https://www.radix-ui.com/)
- Icons from [Lucide](https://lucide.dev/)

---

<div align="center">
  <p>Made with ❤️ by the Vibe Mate Team</p>
  <p>
    <a href="https://github.com/yourusername/vibe-mate/issues">Report Bug</a>
    ·
    <a href="https://github.com/yourusername/vibe-mate/issues">Request Feature</a>
  </p>
</div>
