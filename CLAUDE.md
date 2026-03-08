# The Oracle — Development Guide

## Code Quality

Follow Uncle Bob Martin's Clean Code principles and Sandi Metz's TRUE heuristics
(Transparent, Reasonable, Usable, Exemplary).

## Design Principles

- **Single Responsibility**: every module, struct, and function does one thing
- **Composability**: prefer small, focused pieces that combine cleanly
- **Loose coupling**: depend on abstractions (traits), not concretions
- **High cohesion**: keep related behaviour together

## Style

- Readability wins over terse code
- Self-documenting names — no abbreviations
- Boolean predicates use the `is_` prefix (e.g. `is_archived`, `is_fullscreen`)
- If you need a comment to explain what a function does, extract it into a named function instead

## Functions

- Each function does one thing
- Extract helper functions freely; name them to reveal intent
- Keep parameter lists short; prefer structs when there are more than three

## Testing

- All new code must have tests
- Test behaviour, not implementation details
- Use in-memory databases for persistence tests
- Use wiremock for HTTP provider tests

## Abstractions

- No over-engineering
- No speculative abstractions — only abstract when you have two or more concrete cases
- Remove dead code; do not keep backwards-compatibility hacks

## Error Handling

- Validate at system boundaries only (commands, file I/O, network)
- Propagate errors with `?`; use `anyhow` for internal results and `thiserror` for public error types
- Tauri commands return `Result<T, String>` to satisfy the IPC boundary

## Tech Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2.x |
| Backend | Rust |
| Frontend | React + TypeScript |
| State management | Zustand |
| Styling | Tailwind CSS |
| Database | SQLite (via rusqlite, bundled) |

## Running Tests

```bash
# Frontend
npm test

# Rust unit + integration tests
cargo test

# Rust linter
cargo clippy
```

## Architecture

```
commands → services → domain / providers / persistence
```

- **commands**: thin Tauri IPC handlers; no business logic
- **services**: orchestrate domain objects, providers, and repositories
- **domain**: pure value objects and enums; no I/O
- **providers**: external API integrations (LLM providers)
- **persistence**: SQLite repositories
- **keychain**: OS secure storage for API keys

## Extensibility

| Extension point | How to extend |
|---|---|
| New RPG system | Drop a `.yaml` file in `rpg-systems/` |
| New LLM provider | Implement the `LlmProvider` trait in `src-tauri/src/providers/` and register in `LlmService::new` |
| New UI theme | Add a CSS file and expose the theme name in settings |
