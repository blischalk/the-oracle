# The Oracle

An AI-powered solo RPG desktop application built with Tauri 2, React, TypeScript, and Rust.

## Prerequisites

Install the following before running the app:

- **Rust** (stable) — [rustup.rs](https://rustup.rs)
- **Node.js** (v18+) — [nodejs.org](https://nodejs.org)
- **Tauri system dependencies** — follow the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/) for your OS (macOS, Linux, or Windows)

## Running Locally

```bash
# Install Node dependencies
npm install

# Start the app in development mode (hot-reload for both frontend and backend)
npm run tauri dev
```

The first run will compile the Rust backend, which may take a few minutes.

## Configuration

API keys for LLM providers (OpenAI, Anthropic, Google Gemini, etc.) are stored securely in the OS keychain. Enter them through the app's Settings screen — no `.env` file is required.

## Running Tests

```bash
# Frontend tests
npm test

# Rust unit + integration tests
cargo test --manifest-path src-tauri/Cargo.toml

# Rust linter
cargo clippy --manifest-path src-tauri/Cargo.toml
```

## Building for Production

```bash
npm run tauri build
```

The output installer/bundle will be in `src-tauri/target/release/bundle/`.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) with the following extensions:

- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
