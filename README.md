# SafeShare

Offline desktop app for sanitizing PII from text and files. Built with **Tauri 2 (Rust)** on the backend and **React + TypeScript + Tailwind CSS** on the frontend. **No network calls anywhere.**

## Layout

```
SafeShare/
├── src/                          # React + TS + Tailwind frontend
│   ├── App.tsx                   # Root layout (sidebar + active screen)
│   ├── main.tsx                  # Vite entry
│   ├── index.css                 # Tailwind layer + design tokens
│   ├── components/
│   │   ├── Sidebar.tsx           # Nav: Sanitize / Restore / History / Settings
│   │   └── EmptyState.tsx        # Reusable empty-state card
│   ├── screens/
│   │   ├── SanitizeScreen.tsx
│   │   ├── RestoreScreen.tsx
│   │   ├── HistoryScreen.tsx
│   │   └── SettingsScreen.tsx
│   ├── lib/
│   │   ├── bridge.ts             # Thin wrapper around `invoke()`
│   │   └── nav.ts                # ScreenKey type + ordering
│   └── types/
│       ├── engine.ts             # EngineInfo shape (mirrors Rust)
│       └── tauri.d.ts
├── src-tauri/                    # Tauri (Rust) backend — workspace root
│   ├── Cargo.toml                # Workspace: [safeshare, pii_engine]
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/                    # Replace with real icons before bundling
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs                # `run()` + #[tauri::command] handlers
│   └── pii_engine/               # Workspace member — empty engine
│       ├── Cargo.toml
│       └── src/lib.rs            # engine_info() stub + unit test
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
└── postcss.config.js
```

## How the pieces talk

The frontend **never** makes HTTP requests. Every call to the Rust side goes through Tauri's local IPC:

```ts
// src/lib/bridge.ts
import { invoke } from "@tauri-apps/api/core";
await invoke("engine_info");
```

```rust
// src-tauri/src/lib.rs
#[tauri::command]
fn engine_info() -> pii_engine::EngineInfo {
    pii_engine::engine_info()
}
```

The `Settings` screen calls `engine_info` on mount to prove the wiring works — it returns the engine name and `Cargo` package version.

## Workspace

`src-tauri/Cargo.toml` declares a workspace with two members:

- `safeshare` — the Tauri binary, depends on `pii_engine` via `path = "./pii_engine"`.
- `pii_engine` — currently just exposes `engine_info()`. Real detection / redaction logic lands here next.

## Prerequisites

- **Rust** (stable) + `cargo`
- **Node** ≥ 18
- Tauri 2 platform deps for your OS — see <https://v2.tauri.app/start/prerequisites/>

## Run in development

```bash
npm install
npm run tauri dev
```

## Build a release bundle

```bash
npm run tauri build
```

Before the first release build, drop real icons into `src-tauri/icons/` (the scaffold ships with placeholders so the directory exists; replace them with the platform-correct formats).

## Roadmap

- [ ] PII detection rules (emails, phones, SSNs, cards, names, addresses)
- [ ] Restore-key generation & secure local storage
- [ ] File ingest (drag-and-drop, `.txt` / `.md` / `.csv`)
- [ ] History persistence (SQLite via `tauri-plugin-sql`)
- [ ] Theming + per-rule redaction strategy in Settings