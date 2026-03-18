<div align="center">

# CC Notify

### Notification Manager for AI CLI Tools

[![Version](https://img.shields.io/badge/version-0.1.1-blue.svg)](https://github.com/caterpi11ar/cc-notify/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/caterpi11ar/cc-notify/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)
[![Tests](https://img.shields.io/badge/tests-163%20passing-brightgreen.svg)]()

English | [中文](README_ZH.md) | [日本語](README_JA.md)

</div>

## Why CC Notify?

AI CLI tools like Claude Code, Codex, and Gemini CLI often run long tasks in the background — but you have no way to know when they finish, encounter errors, or need your input, unless you keep staring at the terminal.

**CC Notify** solves this by hooking into these CLI tools and routing notifications to the channels you actually use: system notifications, Slack, Discord, Telegram, Feishu, webhooks, sound alerts, voice readout, and dock badge counts.

- **Never Miss a Task Completion** — Get notified the instant your AI agent finishes, errors out, or needs permission
- **Multi-Channel Routing** — Route different events to different channels (e.g., errors → Slack, completions → native notification)
- **Custom Rules** — Keyword, regex, and file change triggers for fine-grained notification control
- **Webhook Templates** — Built-in support for Feishu (Lark) with extensible template system for future integrations
- **Standalone CLI** — Works without the desktop app via `cc-notify` CLI, perfect for SSH sessions and headless servers
- **Cross-Platform** — Native desktop app for Windows, macOS, and Linux, built with Tauri 2

## Screenshots

> Coming soon — the app is in early development.

## Features

### Notification Channels

| Channel | Description |
|---------|-------------|
| **System Native** | OS-level notifications via Tauri notification plugin |
| **Slack** | Webhook-based messages with channel and mention support |
| **Discord** | Rich embed notifications with custom username and colors |
| **Microsoft Teams** | Incoming webhook connector support |
| **Telegram** | Bot API with configurable parse mode (HTML/Markdown) |
| **HTTP Webhook** | Generic webhook with custom method, headers, and body template |
| **Webhook: Feishu** | Feishu (Lark) bot webhook with interactive card format |
| **Sound** | Cross-platform sound alerts (macOS/Linux/Windows) |
| **Voice** | Text-to-speech or custom local voice pack (macOS) |
| **Tray Badge** | macOS dock badge count that increments with each notification |

### Custom Voice Pack (macOS)

You can configure the `voice` channel in **Channels** with:

- `mode`: `tts` or `voice_pack`
- `voice_pack_dir`: local directory path (required for `voice_pack`)
- `voice` / `rate`: used in `tts` mode

Voice pack naming uses event IDs:

```text
<voice_pack_dir>/
  stop.wav
  notification.idle_prompt.aiff
  notification.permission_prompt.m4a
  default.mp3
```

Lookup order per event:
1. `<event_id>.wav` -> `.aiff` -> `.m4a` -> `.mp3`
2. `default.wav` -> `.aiff` -> `.m4a` -> `.mp3`
3. If still not found, skip playback (no sound)

### Event Types & Routing

- **12 built-in event types** across 3 categories:
  - **Claude Code Hooks**: Task Complete, Idle Prompt, Permission Request, Auth Success, MCP Input, Subagent Stop, Session Start/End
  - **Extended Events**: Long Running, Error, Token Threshold, Cost Threshold
- **Event → Channel routing** with priority levels
- **Per-event enable/disable** toggle

### Custom Rules

- **Keyword Match** — Trigger on specific words in output
- **Regex Match** — Pattern-based triggers
- **File Change** — Glob-based file system triggers
- **Custom Event** — User-defined event types

### Standalone CLI

The `cc-notify` CLI binary works independently — no desktop app required:

```bash
# Send a test notification
cc-notify test

# Triggered automatically via Claude Code hooks
# (installed via Settings → Hooks Integration)
```

### i18n

Full localization in **English**, **Chinese (简体中文)**, and **Japanese (日本語)**.

## Quick Start

### Prerequisites

- **Node.js** 18+
- **pnpm** 8+
- **Rust** 1.85+
- **Tauri CLI** 2.8+

### Install & Run

```bash
# Clone the repository
git clone https://github.com/caterpi11ar/cc-notify.git
cd cc-notify

# Install dependencies
pnpm install

# Run in development mode
pnpm dev
```

### Basic Usage

1. **Channels** — Add notification channels (Slack, Discord, Telegram, etc.) and toggle them on/off
2. **Events** — Configure which events route to which channels
3. **Rules** — Set up custom triggers (keyword, regex, file change)
4. **Hooks** — Install hooks into Claude Code / Codex / Gemini CLI via Settings page
5. **History** — View notification delivery history with status tracking

### Install CLI Hooks

Go to **Settings → Hooks Integration** and click "Install" for each AI CLI tool you use. This installs event hooks that automatically trigger notifications.

## Data Storage

| Location | Content |
|----------|---------|
| `~/.cc-notify/cc-notify.db` | SQLite database (channels, events, rules, routing, history, settings) |
| `~/.cc-notify/disabled` | Kill switch file (touch to disable all notifications) |
| `~/.cc-notify/last_notification` | Rate limiting timestamp |

<details>
<summary><strong>Architecture Overview</strong></summary>

### Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React + TS)                     │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │ Components  │  │   Queries    │  │  TanStack Query  │   │
│  │   (UI)      │──│ (Mutations)  │──│   (Cache/Sync)   │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │ Tauri IPC (invoke)
┌────────────────────────▼────────────────────────────────────┐
│                  Backend (Tauri 2 + Rust)                    │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐   │
│  │  Commands   │  │   Services   │  │  Channel Adapters │   │
│  │ (IPC Layer) │──│ (Bus. Logic) │──│  (Send/Test/Val.) │   │
│  └─────────────┘  └──────────────┘  └──────────────────┘   │
│  ┌─────────────┐  ┌──────────────┐                          │
│  │  Database   │  │   Models     │                          │
│  │ (SQLite DAO)│──│  (Serde)     │                          │
│  └─────────────┘  └──────────────┘                          │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│              Standalone CLI (cc-notify-cli)                  │
│  Same SQLite DB, synchronous reqwest, notify-rust fallback  │
└─────────────────────────────────────────────────────────────┘
```

**Core Design Patterns**

- **SSOT**: All data in `~/.cc-notify/cc-notify.db` (SQLite)
- **Schema Migrations**: Versioned migrations (v1→v3) with `PRAGMA user_version`
- **Channel Adapter Pattern**: `NotificationChannel` trait with `validate_config()`, `send()`, `test()`
- **Template Dispatch**: Webhook channel dispatches by `config.template` (generic, feishu, extensible)
- **Concurrency Safe**: `Mutex<Connection>` with `lock_conn!` macro
- **Layered Architecture**: Commands → Services → DAO → Database

</details>

<details>
<summary><strong>Development Guide</strong></summary>

### Commands

```bash
# Install dependencies
pnpm install

# Dev mode (hot reload)
pnpm dev

# Run frontend unit tests (163 tests)
pnpm test:unit

# Cargo check (Tauri backend)
cargo check --manifest-path src-tauri/Cargo.toml

# Cargo check (CLI)
cargo check --manifest-path cc-notify-cli/Cargo.toml

# Build application
pnpm build
```

### Rust Backend

```bash
cd src-tauri

# Format
cargo fmt

# Lint
cargo clippy

# Test
cargo test
```

### Testing

- **Framework**: vitest
- **Mocking**: MSW (Mock Service Worker) for Tauri IPC calls
- **Components**: @testing-library/react + user-event
- **Coverage**: 15 test files, 163 tests passing

```bash
# Run all tests
pnpm test:unit

# Watch mode
pnpm test:unit -- --watch
```

### Tech Stack

**Frontend**: React 18 · TypeScript · Vite · TailwindCSS · TanStack Query v5 · react-i18next · shadcn/ui · Radix UI · Lucide Icons · Sonner (toasts)

**Backend**: Tauri 2.8 · Rust · rusqlite · reqwest · tokio · serde · chrono · async-trait

**CLI**: Rust · reqwest (blocking) · notify-rust · clap · rusqlite

**Testing**: vitest · MSW · @testing-library/react

</details>

<details>
<summary><strong>Project Structure</strong></summary>

```
├── src/                           # Frontend (React + TypeScript)
│   ├── components/
│   │   ├── channels/              # Channel management (CRUD, test, toggle)
│   │   ├── events/                # Event types & routing configuration
│   │   ├── rules/                 # Custom rule management
│   │   ├── history/               # Notification delivery history
│   │   ├── settings/              # App settings (hooks, quiet hours, etc.)
│   │   ├── common/                # Shared components (ConfirmDialog)
│   │   └── ui/                    # shadcn/ui primitives (15 components)
│   ├── lib/
│   │   ├── api/                   # Tauri IPC wrappers (type-safe)
│   │   └── query/                 # TanStack Query hooks & mutations
│   ├── i18n/locales/              # Translations (en/zh/ja)
│   └── types/                     # TypeScript interfaces
├── src-tauri/                     # Tauri Backend (Rust)
│   └── src/
│       ├── channels/              # Notification channel adapters (10 channels)
│       │   ├── traits.rs          # NotificationChannel trait
│       │   ├── registry.rs        # Channel registry with legacy mapping
│       │   ├── webhook.rs         # Generic + Feishu template dispatch
│       │   ├── native.rs          # System notifications
│       │   └── ...                # slack, discord, teams, telegram, sound, voice, tray_badge
│       ├── commands/              # Tauri IPC command handlers
│       ├── services/              # Business logic layer
│       ├── database/              # SQLite schema, migrations, DAO
│       ├── hooks/                 # AI tool hook generators (Claude, Codex, Gemini)
│       └── models/                # Data models (Channel, Rule, Routing, etc.)
├── cc-notify-cli/                 # Standalone CLI binary
│   └── src/
│       ├── main.rs                # CLI entry (clap)
│       ├── notify.rs              # Notification pipeline & channel dispatch
│       ├── hooks.rs               # Hook installer
│       └── db.rs                  # SQLite helpers
├── tests/                         # Frontend test suite
│   ├── components/                # Component tests (5 pages)
│   ├── api/                       # API layer tests (8 modules)
│   ├── query/                     # Query/mutation tests
│   └── msw/                       # Mock handlers & state
└── public/                        # Static assets
```

</details>

## FAQ

<details>
<summary><strong>Which AI CLI tools are supported?</strong></summary>

CC Notify supports **Claude Code**, **Codex**, and **Gemini CLI**. Hook installers are available in Settings → Hooks Integration.

</details>

<details>
<summary><strong>Does the CLI work without the desktop app?</strong></summary>

Yes. The `cc-notify` CLI binary reads the same SQLite database and sends notifications independently. If no database exists, it falls back to native system notifications.

</details>

<details>
<summary><strong>How does the Feishu (Lark) integration work?</strong></summary>

Feishu is implemented as a **webhook template** — when creating a webhook channel, select the "Feishu" template. The backend will use the Feishu bot API format (interactive card in CLI, text message in Tauri) and validate the webhook URL against Feishu/Larksuite domains.

</details>

<details>
<summary><strong>Why doesn't `window.confirm()` work?</strong></summary>

Tauri's WebView doesn't support `window.confirm()` — it auto-confirms without showing a dialog. CC Notify uses a custom `ConfirmDialog` component (built on Radix AlertDialog) that works correctly in the Tauri environment.

</details>

<details>
<summary><strong>How do I reset the tray badge count?</strong></summary>

The badge count resets when you restart the app. You can also disable the Tray Badge channel from the Channels page to stop badge updates entirely.

</details>

<details>
<summary><strong>macOS: "CC Notify" cannot be opened / unidentified developer</strong></summary>

The app is not signed with an Apple Developer certificate. macOS Gatekeeper blocks apps downloaded from the internet that are unsigned.

**Option 1** — Right-click the app → select "Open" → click "Open" in the dialog (only needed once).

**Option 2** — Go to **System Settings → Privacy & Security**, scroll down and click **"Open Anyway"**.

**Option 3** — Remove the quarantine flag via Terminal:

```bash
xattr -cr "/Applications/CC Notify.app"
```

After any of these steps, the app will open normally going forward.

</details>

## Contributing

Issues and pull requests are welcome!

Before submitting a PR:

```bash
# Ensure all checks pass
cargo check --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path cc-notify-cli/Cargo.toml
pnpm test:unit
```

## License

MIT
