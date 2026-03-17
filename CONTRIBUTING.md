# Contributing to infs

Thank you for your interest in contributing! This document covers development setup, coding conventions, and the release process.

## Table of Contents

- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Running Tests](#running-tests)
- [Commit Message Conventions](#commit-message-conventions)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Release Process](#release-process)
- [Adding a New Provider](#adding-a-new-provider)

## Development Setup

**Requirements:** Rust 1.75 or later. Install via [rustup](https://rustup.rs).

```bash
git clone https://github.com/dvaJi/infera
cd infera
cargo build          # debug build
cargo build --release  # optimised binary → target/release/infs
```

No additional tooling is required for a basic development loop.

## Project Structure

```
src/
├── main.rs              # Binary entry point — wires CLI to commands
├── error.rs             # InfsError enum (thiserror-based)
├── types.rs             # Shared types: AppDescriptor, RunResponse, ProviderDescriptor, …
├── config/              # Config + credentials load/save (TOML)
├── auth/                # AuthMethod abstractions
├── providers/           # Provider trait, registry, and per-provider adapters
│   ├── mod.rs           # Provider async trait
│   ├── registry.rs      # ProviderRegistry::build_registry()
│   ├── openrouter.rs    # ✅ Reference implementation
│   ├── falai.rs         # ✅ Full implementation (image, async queue)
│   ├── replicate.rs     # ✅ Full implementation (image, prediction polling)
│   └── wavespeed.rs     # ✅ Full implementation
├── catalog/             # Aggregates listings across providers
└── cli/                 # Clap-based subcommands
    ├── mod.rs
    ├── provider.rs      # provider list/connect/show/disconnect
    ├── app.rs           # app list/run/show
    ├── config.rs        # config path
    └── doctor.rs        # doctor
```

## Running Tests

```bash
cargo test
```

All tests must pass before opening a PR. To run a specific test or module:

```bash
cargo test <test_name>
cargo test providers::
```

Set `RUST_LOG=debug` to see detailed log output during test runs.

## Commit Message Conventions

This project follows [Conventional Commits](https://www.conventionalcommits.org/). Commit messages are parsed automatically by [Release Please](#release-process) to generate changelogs and determine the next semantic version.

### Format

```
<type>(<optional scope>): <short description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Changelog section | SemVer bump |
|---|---|---|
| `feat` | Features | minor |
| `fix` | Bug Fixes | patch |
| `perf` | Performance Improvements | patch |
| `deps` | Dependencies | patch |
| `revert` | Reverts | patch |
| `docs` | Documentation | — |
| `style` | *(hidden)* | — |
| `chore` | *(hidden)* | — |
| `refactor` | *(hidden)* | — |
| `test` | *(hidden)* | — |
| `build` | *(hidden)* | — |
| `ci` | *(hidden)* | — |

A `!` suffix (e.g. `feat!: …`) or a `BREAKING CHANGE:` footer triggers a **major** version bump.

### Examples

```
feat(providers): add ElevenLabs TTS provider
fix(config): handle missing credentials.toml gracefully
docs: add curl install snippet to README
chore: update dependencies
```

## Pull Request Guidelines

1. Branch from `main` and open your PR against `main`.
2. Keep PRs focused — one logical change per PR.
3. Make sure `cargo test` passes locally.
4. Use Conventional Commit messages; Release Please reads the commits merged into `main` to build the changelog.
5. Update `README.md` if you change user-visible behaviour.
6. Add or update tests for non-trivial logic changes.

## Release Process

Releases are fully automated using two GitHub Actions workflows:

### CI (`ci.yml`)

Every pull request and push to `main` triggers the CI workflow, which:

- Builds the project (`cargo build`) on Linux, macOS, and Windows.
- Runs the full test suite (`cargo test`).
- Checks formatting (`cargo fmt --check`).
- Runs Clippy lints (`cargo clippy -- -D warnings`).

All checks must pass before merging.

### 1. Release Please (`release-please.yml`)

Every time a commit is merged to `main`, [Release Please](https://github.com/googleapis/release-please) inspects the commit history, then:

- Opens (or updates) a **Release PR** that bumps `Cargo.toml` / `Cargo.lock` version numbers and adds a new section to `CHANGELOG.md`.
- When that Release PR is merged, it automatically creates a GitHub Release with the generated changelog as the release body.

### 2. Binary builds (`release.yml`)

When a GitHub Release is published (triggered by merging the Release Please PR), a second workflow:

- Builds the `infs` binary for all five supported targets in parallel.
- Uploads each binary as a named release asset.

| Asset | Target |
|---|---|
| `infs-linux-x86_64` | `x86_64-unknown-linux-musl` (static) |
| `infs-linux-aarch64` | `aarch64-unknown-linux-musl` (static) |
| `infs-macos-x86_64` | `x86_64-apple-darwin` |
| `infs-macos-aarch64` | `aarch64-apple-darwin` |
| `infs-windows-x86_64.exe` | `x86_64-pc-windows-msvc` |

### Summary

```
commit merged to main
       │
       ▼
Release Please opens / updates Release PR
       │
       ▼  (Release PR merged)
GitHub Release created with changelog
       │
       ▼
Binary build workflow runs → release assets uploaded
```

**You do not need to manually tag, create releases, or bump version numbers.** Just merge commits with the correct Conventional Commit types and Release Please handles the rest.

## Adding a New Provider

See the [Adding a new provider](README.md#adding-a-new-provider) section in the README for step-by-step instructions.

Key points:

- Implement the `Provider` async trait in `src/providers/<name>.rs`.
- Register the provider in `src/providers/registry.rs`.
- Add unit tests that mock HTTP calls where possible.
- Follow the OpenRouter or WaveSpeed implementations as reference examples.
