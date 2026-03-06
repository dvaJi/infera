# infs

A fast, provider-agnostic CLI for discovering, connecting, and executing AI apps and models from multiple providers through one consistent interface.

## What is infs?

`infs` lets you run AI models from multiple providers using a single, unified command-line interface. Instead of learning each provider's SDK or API, you use one tool with a consistent command structure.

```
infs app run openrouter/anthropic/claude-sonnet-4-5 --input '{"prompt":"Explain quantum computing"}'
infs app run falai/fal-ai/flux-dev-lora --input '{"prompt":"a cat astronaut in space"}'
infs app list --category image
```

## Why this exists

Every AI provider has its own API shape, auth flow, and SDK. `infs` wraps them behind a single interface so developers and coding agents can:

- **Discover** what models are available across providers
- **Connect** to providers with a single interactive command
- **Run** any model using a consistent input/output contract
- **Script** AI workflows without provider-specific boilerplate

## Current Status

| Provider | Category | Status | Auth |
|---|---|---|---|
| OpenRouter | LLM | ✅ Fully implemented | API key |
| fal.ai | Image | 🚧 Scaffolded (listing works, run not yet implemented) | API key |
| Replicate | Image | 🚧 Scaffolded (listing works, run not yet implemented) | API key |
| WaveSpeed AI | Image | 🚧 Scaffolded (listing works, run not yet implemented) | API key |

**OpenRouter** is the reference implementation: full end-to-end API key auth and model execution work today.

The image providers (fal.ai, Replicate, WaveSpeed) are registered and appear in listings, but execution returns a "not yet implemented" error. The architecture makes it straightforward to add execution — see the [Development](#development) section.

## Installation

### Build from source

```bash
git clone https://github.com/dvaJi/infera
cd infera
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
infs app run falai/fal-ai/flux-dev-lora --input '{"prompt":"a cat astronaut in space"}'
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
**Status:** Scaffolded — `app list` works, `app run` is not yet implemented.

### Replicate

Replicate runs machine learning models in the cloud.

**Website:** https://replicate.com  
**Status:** Scaffolded — `app list` works, `app run` is not yet implemented.

### WaveSpeed AI

WaveSpeed AI provides fast image generation.

**Website:** https://wavespeed.ai  
**Status:** Scaffolded — `app list` works, `app run` is not yet implemented.

## Authentication

`infs` stores configuration in your OS's standard config directory:

- **Linux:** `~/.config/infs/`
- **macOS:** `~/Library/Application Support/infs/`
- **Windows:** `%APPDATA%\infs\`

Two files are used:
- `config.toml` — provider settings and metadata (non-sensitive)
- `credentials.toml` — API keys and secrets

> **Note:** Credentials are stored in a plain TOML file for now. A future version will integrate with the OS keychain via the `keyring` crate. The credential storage is isolated behind a trait interface to make this swap straightforward.

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

pub struct MyProvider { /* ... */ }

#[async_trait]
impl Provider for MyProvider {
    fn descriptor(&self) -> &ProviderDescriptor { /* ... */ }
    fn supported_auth_methods(&self) -> Vec<AuthMethod> { /* ... */ }
    fn list_apps(&self) -> Vec<AppDescriptor> { /* ... */ }
    async fn run_app(&self, app_id: &str, input: Value, config: &ProviderConfig) -> Result<RunResponse, InfsError> { /* ... */ }
    fn validate_config(&self, config: &ProviderConfig) -> Result<(), InfsError> { /* ... */ }
}
```

3. Register it in `src/providers/registry.rs`:

```rust
pub fn build_registry() -> ProviderRegistry {
    ProviderRegistry::new(vec![
        // ...existing providers...
        Box::new(myprovider::MyProvider::new()),
    ])
}
```

### Completing a scaffolded provider

The image providers (fal.ai, Replicate, WaveSpeed) are ready to implement. Each file has a `run_app` stub returning `InfsError::NotImplemented`. To complete them:

1. Read the provider's API documentation
2. Add the request/response types
3. Implement the HTTP call in `run_app`
4. Add tests

See `src/providers/openrouter.rs` as the reference implementation.

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
- Built-in model catalog is static — dynamic discovery from provider APIs is not yet implemented
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
- [ ] Dynamic model discovery from provider APIs
- [ ] File output for image generation (download to local file)
- [ ] File input support
- [ ] Retry logic with exponential backoff
- [ ] Shell completion scripts (`infs completions bash`)
- [ ] More providers (ElevenLabs, Stability AI, etc.)
- [ ] OAuth support for providers that require it

## License

MIT