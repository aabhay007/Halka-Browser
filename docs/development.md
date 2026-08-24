# Development & Build Guide

## Setup Environment

Ensure Rust, CMake, and Node.js are installed:
```bash
cargo --version
rustc --version
cmake --version
node --version
```

---

## Workspace Setup

Clone the repository and run:
```bash
# Check rust compilation
cargo check --manifest-path src-tauri/Cargo.toml

# Execute unit tests
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## Running in Debug Mode

```bash
cargo build --manifest-path src-tauri/Cargo.toml
./src-tauri/target/debug/halka_browser.exe
```

DevTools will open automatically in debug mode (`#[cfg(debug_assertions)]`).
