# Antigravity Desktop Chromium Browser

A modern, cross-platform desktop web browser shell built around a native Chromium engine runtime.

> **Absolute Architecture Rule:** External websites are rendered exclusively inside native browser engine surfaces (`WebView2` on Windows). Iframes are strictly **never** used for website display.

---

## Key Features

- **Genuine Chromium Engine**: Uses Microsoft Edge `WebView2` Evergreen Chromium engine on Windows.
- **Native Multi-Tab Architecture**: Each open tab owns an independent native OS webview surface.
- **No-Iframe Bounded Chrome**: Browser Chrome toolbar (tabs, address bar, buttons) is physically separated from web content.
- **SQLite Persistence**: History, Bookmarks, and Settings stored locally in `.browser_data/browser_data.db`.
- **Keyboard Shortcuts**: Full Chrome-like shortcut suite (`Ctrl+T`, `Ctrl+W`, `Ctrl+Shift+T`, `Ctrl+Tab`, `Ctrl+L`, `Ctrl+D`, `Ctrl+R`, `Ctrl+Shift+I`).
- **DevTools**: Integrated native Chromium Developer Tools (`open_devtools`).

---

## Project Structure

```
d:/web2/
├── docs/
│   ├── architecture-decision.md
│   ├── architecture.md
│   ├── browser-engine.md
│   ├── development.md
│   ├── testing.md
│   └── platform-support.md
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs
│   │   ├── database/
│   │   │   └── mod.rs
│   │   └── browser_core/
│   │       ├── bookmarks.rs
│   │       ├── downloads.rs
│   │       ├── history.rs
│   │       ├── navigation.rs
│   │       ├── settings.rs
│   │       └── tab_manager.rs
│   └── icons/
└── ui/
    ├── index.html
    ├── newtab.html
    ├── css/
    │   └── style.css
    └── js/
        └── chrome.js
```

---

## Getting Started & Running the Project

### Prerequisites
- **Rust & Cargo** (1.78+ recommended): [Install Rust](https://rustup.rs/)
- **C++ Build Tools / CMake**: (On Windows: Visual Studio C++ Build Tools)
- **WebView2 Runtime**: (Pre-installed on Windows 10/11)
- *(Optional)* **Node.js**: If using Tauri CLI (`@tauri-apps/cli`)

---

### Quick Start (Run Directly)

#### Option 1: Cargo (Recommended)
From the project root:
```bash
# Compiles and launches the browser in debug mode
cargo run --manifest-path src-tauri/Cargo.toml
```

Or navigate to `src-tauri` first:
```bash
cd src-tauri
cargo run
```

#### Option 2: Tauri CLI
```bash
# Using cargo-tauri
cargo tauri dev

# Or using npx
npx @tauri-apps/cli dev
```

---

### Building the Project

#### Development (Debug) Build
```bash
# 1. Verify compilation
cargo check --manifest-path src-tauri/Cargo.toml

# 2. Build debug executable
cargo build --manifest-path src-tauri/Cargo.toml

# 3. Launch compiled binary
# On Windows (PowerShell):
.\src-tauri\target\debug\browser_app.exe

# On Windows (Command Prompt):
src-tauri\target\debug\browser_app.exe

# On Linux / macOS:
./src-tauri/target/debug/browser_app
```

#### Production (Release) Build
```bash
# Build optimized release executable
cargo build --release --manifest-path src-tauri/Cargo.toml

# Launch release binary:
# Windows (PowerShell):
.\src-tauri\target\release\browser_app.exe

# Linux / macOS:
./src-tauri/target/release/browser_app
```

---

### Running Tests
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## Documentation

- [Architecture Overview](docs/architecture.md)
- [Architecture Decision Document](docs/architecture-decision.md)
- [Browser Engine Integration](docs/browser-engine.md)
- [Development Guide](docs/development.md)
- [Testing Guide](docs/testing.md)
- [Platform Support Matrix](docs/platform-support.md)
