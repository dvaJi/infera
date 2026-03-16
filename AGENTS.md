# AGENTS.md

> This file follows the [agents.md](https://agents.md/) specification. It gives AI coding agents the context they need to work effectively in this repository.

## Project Overview

**`infs`** is a fast, provider-agnostic CLI for discovering, connecting to, and running AI apps and models from multiple cloud providers through one consistent interface.

- **Language:** Rust 2021 edition
- **Binary name:** `infs`
- **Crate name:** `infs` (see `Cargo.toml`)
- **Minimum Rust version:** 1.75

## Build & Test Commands

```bash
# Build (debug)
cargo build

# Build (optimised release binary → target/release/infs)
cargo build --release

# Run all tests
cargo test

# Run a specific test
cargo test <test_name>

# Run tests for a module
cargo test providers::

# Enable debug logging during tests
RUST_LOG=debug cargo test
```

All tests must pass before submitting any change. The expected pass count is in the 30s; if you see fewer tests passing something is broken.

## Repository Layout

```
.
├── Cargo.toml                    # Package manifest (name = "infs", version controlled by Release Please)
├── Cargo.lock
├── release-please-config.json    # Release Please: changelog section definitions
├── .release-please-manifest.json # Release Please: current version tracking
├── README.md
├── CONTRIBUTING.md
├── AGENTS.md                     # ← this file
├── .github/
│   └── workflows/
│       ├── release.yml           # Builds binaries for 5 targets on GitHub Release
│       └── release-please.yml   # Automates changelogs + version bumps on push to main
└── src/
    ├── main.rs                   # Entry point — parses CLI args and dispatches
    ├── error.rs                  # InfsError (thiserror)
    ├── types.rs                  # AppDescriptor, RunResponse, ProviderDescriptor, AuthMethod, …
    ├── config/
    │   └── mod.rs                # Config + credentials load/save; credentials written mode 0600 on Unix
    ├── auth/
    │   └── mod.rs                # AuthMethod enum
    ├── providers/
    │   ├── mod.rs                # Provider async trait
    │   ├── registry.rs           # ProviderRegistry::build_registry()
    │   ├── openrouter.rs         # ✅ Reference implementation (LLM)
    │   ├── falai.rs              # 🚧 Listing works; run_app returns NotImplemented
    │   ├── replicate.rs          # 🚧 Listing works; run_app returns NotImplemented
    │   └── wavespeed.rs          # ✅ Full implementation (image/video)
    ├── catalog/
    │   └── mod.rs                # AppCatalog — aggregates listings across all providers
    └── cli/
        ├── mod.rs                # CLI root; Clap Command enum
        ├── provider.rs           # provider list / connect / show / disconnect
        ├── app.rs                # app list / run / show
        ├── config.rs             # config path
        └── doctor.rs             # doctor
```

## Key Abstractions

### `Provider` trait (`src/providers/mod.rs`)

Every provider implements this async trait:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn descriptor(&self) -> &ProviderDescriptor;
    fn supported_auth_methods(&self) -> Vec<AuthMethod>;
    async fn list_apps(&self, config: &ProviderConfig) -> Result<Vec<AppDescriptor>, InfsError>;
    async fn run_app(&self, app_id: &str, input: serde_json::Value, config: &ProviderConfig) -> Result<RunResponse, InfsError>;
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError>;
}
```

### `ProviderDescriptor` (`src/types.rs`)

Contains `id`, `name`, `description`, `website`, and **`api_key_help_url`**. Always use `d.api_key_help_url` in the CLI — do **not** construct `{website}/keys` manually.

### App IDs

The full app ID used on the command line is `<provider_id>/<app_id>`, e.g. `openrouter/anthropic/claude-sonnet-4-5`.  
`AppDescriptor::full_id()` returns this prefixed form. `Provider::run_app` receives only the provider-local part (everything after the first `/`).

### Config & Credentials

- Config directory: resolved by the `directories` crate (`ProjectDirs`).
- `config.toml` — non-sensitive settings; `credentials.toml` — API keys.
- `save_config` deliberately strips credentials before writing `config.toml`.
- On Unix, `credentials.toml` is created with mode `0o600`.

## Common Patterns

### Adding a New Provider

1. Create `src/providers/myprovider.rs` — implement the `Provider` trait.
2. Register in `src/providers/registry.rs` inside `build_registry()`.
3. Add unit tests (mock HTTP responses where practical).
4. Use `openrouter.rs` (LLM) or `wavespeed.rs` (async poll pattern) as a reference.

### Error Handling

Use `InfsError` variants (`src/error.rs`) for all errors that propagate to the CLI. `anyhow` is used internally in functions that never surface to end users.

### HTTP Requests

Use `reqwest` with the `rustls-tls` feature (no OpenSSL dependency). The client is constructed per-provider.

## Commit & PR Conventions

This repo follows [Conventional Commits](https://www.conventionalcommits.org/):

```
feat:     new user-visible feature          → minor version bump
fix:      bug fix                           → patch bump
perf:     performance improvement           → patch bump
deps:     dependency update                 → patch bump
docs:     documentation only               → no bump
chore:    maintenance (no public API change)→ no bump
feat!:    breaking change                  → major bump
```

Release Please reads these commit messages to auto-generate `CHANGELOG.md` and version bumps. See `CONTRIBUTING.md` for the full release workflow.

## CI / Release Workflow

| Workflow | Trigger | What it does |
|---|---|---|
| `release-please.yml` | push to `main` | Opens/updates a Release PR; on merge creates a GitHub Release |
| `release.yml` | GitHub Release published | Builds `infs` for 5 targets; uploads binaries as release assets |

Binary targets built on each release:

| Asset name | Target triple |
|---|---|
| `infs-linux-x86_64` | `x86_64-unknown-linux-musl` |
| `infs-linux-aarch64` | `aarch64-unknown-linux-musl` |
| `infs-macos-x86_64` | `x86_64-apple-darwin` |
| `infs-macos-aarch64` | `aarch64-apple-darwin` |
| `infs-windows-x86_64.exe` | `x86_64-pc-windows-msvc` |

## Things to Avoid

- Do **not** hardcode API endpoints inside test code; use mocks or feature flags.
- Do **not** store secrets in `config.toml`; they belong in `credentials.toml`.
- Do **not** bump versions in `Cargo.toml` manually — Release Please does this automatically.
- Do **not** use `{provider.website}/keys` to construct help URLs; use `descriptor.api_key_help_url`.
