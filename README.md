# infs

A fast, provider-agnostic CLI for discovering, connecting, and executing AI apps and models from multiple providers through one consistent interface.

## What is infs?

`infs` lets you run AI models from multiple providers using a single, unified command-line interface. Instead of learning each provider's SDK or API, you use one tool with a consistent command structure.

```
infs app run openrouter/anthropic/claude-sonnet-4-5 --input '{"prompt":"Explain quantum computing"}'
infs app run falai/fal-ai/flux/dev --input '{"prompt":"a cat astronaut in space"}'
infs app list --category image
```

## Why this exists

Every AI provider has its own API shape, auth flow, and SDK. `infs` wraps them behind a single interface so developers and coding agents can:

- **Discover** what models are available across providers
- **Connect** to providers with a single interactive command
- **Run** any model using a consistent input/output contract
- **Script** AI workflows without provider-specific boilerplate

## Current Status

| Provider | Category | Status | Auth | App Listing |
|---|---|---|---|---|
| OpenRouter | LLM | ✅ Fully implemented | API key | Live from API when connected, static fallback when not |
| fal.ai | Image | 🚧 Listing live, run not yet implemented | API key | Live from `api.fal.ai/v1/models` when connected |
| Replicate | Image | 🚧 Listing live, run not yet implemented | API key | Live from `api.replicate.com/v1/models` when connected |
| WaveSpeed AI | Image/Video | 🚧 Listing live, run not yet implemented | API key | Live from `api.wavespeed.ai/api/v3/models` when connected |

**OpenRouter** is the reference implementation: full end-to-end API key auth and model execution work today. When connected, `app list` fetches live from the OpenRouter API.

The image providers (fal.ai, Replicate, WaveSpeed) support **live model listing** from their APIs when an API key is configured. Execution (`app run`) returns a "not yet implemented" error. When no API key is configured, a static fallback list of well-known models is shown.

## Installation

### Build from source

```bash
git clone https://github.com/dvaJi/infs
cd infs
cargo build --release
# Binary is at ./target/release/infs
```

To install globally:

```bash
cargo install --path .
```

**Requirements:** Rust 1.75+ ([install via rustup](https://rustup.rs))

## Usage

### Provider management

```bash
# List all providers and their connection status
infs provider list

# Connect to a provider (interactive)
infs provider connect openrouter

# Show details for a specific provider
infs provider show openrouter

# Disconnect from a provider
infs provider disconnect openrouter
```

### App/model management

```bash
# List all available apps/models
infs app list

# Filter by category
infs app list --category image
infs app list --category llm

# Filter by provider
infs app list --provider openrouter
infs app list --provider falai

# Show details for a specific app
infs app show openrouter/anthropic/claude-sonnet-4-5
```

### Running apps

```bash
# Run an LLM via OpenRouter
infs app run openrouter/anthropic/claude-sonnet-4-5 --input '{"prompt":"Explain quantum computing"}'

# Run GPT-4o
infs app run openrouter/openai/gpt-4o --input '{"prompt":"Write a haiku about Rust"}'

# Run a free model
infs app run openrouter/meta-llama/llama-3.1-8b-instruct --input '{"prompt":"What is 2+2?"}'

# Image generation (scaffolded — will return not-yet-implemented error)
infs app run falai/fal-ai/flux/dev --input '{"prompt":"a cat astronaut in space"}'
```

### Utilities

```bash
# Show config file location
infs config path

# Check connection status and diagnose issues
infs doctor
```

## Providers

### OpenRouter

OpenRouter provides a unified API for hundreds of LLM models from OpenAI, Anthropic, Google, Meta, Mistral, and many more.

**Website:** https://openrouter.ai  
**Get an API key:** https://openrouter.ai/keys

**Models included in built-in catalog:**
- `openrouter/openai/gpt-4o` — GPT-4o
- `openrouter/openai/gpt-4o-mini` — GPT-4o Mini
- `openrouter/anthropic/claude-sonnet-4-5` — Claude Sonnet 4.5
- `openrouter/google/gemini-flash-1.5` — Gemini Flash 1.5
- `openrouter/meta-llama/llama-3.1-8b-instruct` — Llama 3.1 8B (free)
- `openrouter/mistralai/mistral-7b-instruct` — Mistral 7B (free)

Any model available on OpenRouter can be run using its OpenRouter model ID (e.g., `openrouter/cohere/command-r-plus`).

### fal.ai

fal.ai provides fast, serverless image generation APIs.

**Website:** https://fal.ai  
**Get an API key:** https://fal.ai/dashboard/keys  
**Status:** Live model listing from `https://api.fal.ai/v1/models` when connected. `app run` not yet implemented.

### Replicate

Replicate runs machine learning models in the cloud.

**Website:** https://replicate.com  
**Get an API key:** https://replicate.com/account/api-tokens  
**Status:** Live model listing from `https://api.replicate.com/v1/models` when connected. `app run` not yet implemented.

### WaveSpeed AI

WaveSpeed AI provides fast image and video generation.

**Website:** https://wavespeed.ai  
**Get an API key:** https://wavespeed.ai/  
**Status:** Live model listing from `https://api.wavespeed.ai/api/v3/models` when connected. `app run` not yet implemented.

## Authentication

`infs` stores configuration in your OS's standard config directory:

- **Linux:** `~/.config/infs/`
- **macOS:** `~/Library/Application Support/infs/infs/`
- **Windows:** `%APPDATA%\infs\infs\`

Two files are used:
- `config.toml` — provider settings and metadata (non-sensitive)
- `credentials.toml` — API keys and secrets

> **Note:** On Unix, `credentials.toml` is written with file mode `0600` (owner read/write only). A future version will optionally integrate with the OS keychain via the `keyring` crate.

### Connecting to OpenRouter

```bash
infs provider connect openrouter
```

You will be prompted:

```
Connecting to OpenRouter
Auth method: API Key
? Enter your OpenRouter API key: [hidden]
✓ Connected to OpenRouter
```

## Examples

### Ask a question

```bash
infs provider connect openrouter  # First time only
infs app run openrouter/meta-llama/llama-3.1-8b-instruct --input '{"prompt":"What is the capital of France?"}'
```

### Use advanced message format

OpenRouter also accepts the full messages format:

```bash
infs app run openrouter/openai/gpt-4o --input '{
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "What is Rust?"}
  ]
}'
```

### Browse image models

```bash
infs app list --category image
```

### Check what is configured

```bash
infs doctor
```

## Development

### Architecture

```
src/
├── main.rs              # Entry point
├── error.rs             # InfsError type
├── types.rs             # Shared types (AppId, AppDescriptor, RunResponse, etc.)
├── config/              # Config loading/saving
├── auth/                # Auth method abstractions
├── providers/           # Provider trait + registry + adapters
│   ├── mod.rs           # Provider trait
│   ├── registry.rs      # ProviderRegistry
│   ├── openrouter.rs    # ✅ Full implementation
│   ├── falai.rs         # 🚧 Scaffolded
│   ├── replicate.rs     # 🚧 Scaffolded
│   └── wavespeed.rs     # 🚧 Scaffolded
├── catalog/             # App catalog (aggregates provider listings)
└── cli/                 # CLI commands
    ├── mod.rs
    ├── provider.rs      # provider list/connect/show/disconnect
    ├── app.rs           # app list/run/show
    ├── config.rs        # config path
    └── doctor.rs        # doctor
```

### Adding a new provider

1. Create `src/providers/myprovider.rs`
2. Implement the `Provider` trait:

```rust
use async_trait::async_trait;
use crate::providers::Provider;
use crate::config::ProviderConfig;
use crate::error::InfsError;
use crate::types::{AppDescriptor, AuthMethod, ProviderDescriptor, RunResponse};

pub struct MyProvider { descriptor: ProviderDescriptor }

#[async_trait]
impl Provider for MyProvider {
    fn descriptor(&self) -> &ProviderDescriptor { &self.descriptor }
    fn supported_auth_methods(&self) -> Vec<AuthMethod> { vec![AuthMethod::ApiKey] }
    async fn list_apps(&self, config: &ProviderConfig) -> Result<Vec<AppDescriptor>, InfsError> { /* fetch and return available apps from the provider API */ }
    async fn run_app(&self, app_id: &str, input: serde_json::Value, config: &ProviderConfig) -> Result<RunResponse, InfsError> { /* call the provider's inference API */ }
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError> { /* check that required credentials are present */ }
}
```

3. Register it in `src/providers/registry.rs`:

```rust
pub fn build_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    // ...existing providers...
    registry.register(Box::new(myprovider::MyProvider::new()));
    registry
}
```

### Completing provider execution

The image providers (fal.ai, Replicate, WaveSpeed) already fetch live model listings from their APIs. What remains is execution. Each file has a `run_app` stub returning `InfsError::NotImplemented`. To complete them:

1. Read the provider's API documentation
2. Add the request/response types for execution
3. Implement the HTTP call in `run_app`
4. Add tests

### Running tests

```bash
cargo test
```

### Environment variables

| Variable | Purpose |
|---|---|
| `RUST_LOG` | Log level (`error`, `warn`, `info`, `debug`, `trace`) |

## Known Limitations

- Image provider execution (fal.ai, Replicate, WaveSpeed) is not yet implemented
- Model listing requires an API key for fal.ai, Replicate, and WaveSpeed; a static fallback is shown when not connected
- Credentials are stored in a plain TOML file, not the OS keychain
- No streaming support for LLM responses
- No file input/output for image generation artifacts
- No `--json` output mode yet
- No retry/backoff on network errors

## Roadmap

- [ ] Complete fal.ai execution (image generation via API)
- [ ] Complete Replicate execution
- [ ] Complete WaveSpeed AI execution
- [ ] OS keychain integration for credential storage (`keyring` crate)
- [ ] `--json` output flag for machine-friendly output
- [ ] Streaming LLM responses
- [ ] Paginated model listing for providers returning large catalogs
- [ ] File output for image generation (download to local file)
- [ ] File input support
- [ ] Retry logic with exponential backoff
- [ ] Shell completion scripts (`infs completions bash`)
- [ ] More providers (ElevenLabs, Stability AI, etc.)
- [ ] OAuth support for providers that require it

## License

MIT