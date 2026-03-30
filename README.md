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
| fal.ai | Image | ✅ Listing live + run implemented | API key | Live from `api.fal.ai/v1/models` when connected |
| Replicate | Image | ✅ Listing live + run implemented | API key | Live from `api.replicate.com/v1/models` when connected |
| WaveSpeed AI | Image/Video | ✅ Listing live + run implemented | API key | Live from `api.wavespeed.ai/api/v3/models` when connected |

**OpenRouter** is the reference implementation: full end-to-end API key auth and model execution work today. When connected, `app list` fetches live from the OpenRouter API.

**WaveSpeed AI** is also fully implemented: model listing and image/video generation both work end-to-end. When connected, `app list` fetches live from the WaveSpeed API, and `app run` submits an inference task and polls for the result.

**fal.ai** and **Replicate** are also fully implemented: live model listing and inference both work. `app run` submits a job and polls for the result.

## Installation

### Quick install (recommended)

**macOS / Linux:**

```bash
curl -fsSL https://raw.githubusercontent.com/dvaji/infera/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
iex "& { $(irm https://raw.githubusercontent.com/dvaji/infera/main/install.ps1) }"
```

The installer will download the latest release and place it in `~/.local/bin` (Unix) or `%USERPROFILE%\.local\bin` (Windows). You may need to add this directory to your PATH.

### Download a pre-built binary

Pre-built binaries are attached to every [GitHub Release](https://github.com/dvaJi/infera/releases/latest).
Download the binary for your platform and place it somewhere on your `PATH`:

| Platform | File |
|---|---|
| Linux x86_64 | `infs-linux-x86_64` |
| Linux aarch64 (ARM) | `infs-linux-aarch64` |
| macOS aarch64 (Apple Silicon) | `infs-macos-aarch64` |
| Windows x86_64 | `infs-windows-x86_64.exe` |

**macOS / Linux quick-install example:**

```bash
# Replace <version> and <platform> with the appropriate values
curl -fsSL https://github.com/dvaji/infera/releases/download/<version>/infs-<platform> \
  -o infs
chmod +x infs
sudo mv infs /usr/local/bin/
# Or install to a user-writable path (no sudo required):
# mkdir -p ~/.local/bin && mv infs ~/.local/bin/
```

### Self-update

Once installed, `infs` can update itself:

```bash
# Check for updates
infs self check

# Update to the latest version
infs self update

# Skip confirmation prompt
infs self update --yes
```

### Build from source

```bash
git clone https://github.com/dvaji/infera
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
# List providers and whether they are ready to use
infs app list

# List all models for a specific provider
infs app list openrouter
infs app list falai

# Filter by category
infs app list --category image
infs app list --category llm

# Filter a provider's models by category
infs app list openrouter --category llm

# Paginate results (page number and items per page)
infs app list openrouter --page 2 --per-page 50

# Show details for a specific app
infs app show openrouter/anthropic/claude-sonnet-4-5
```

`infs app list` has two modes:

- Without a provider, it shows providers with status like `available` or `needs credentials`
- With a provider argument, it lists that provider's models and supports pagination (`--page` and `--per-page` flags)

### Running apps

```bash
# Run an LLM via OpenRouter
infs app run openrouter/anthropic/claude-sonnet-4-5 --input '{"prompt":"Explain quantum computing"}'

# Run GPT-4o
infs app run openrouter/openai/gpt-4o --input '{"prompt":"Write a haiku about Rust"}'

# Run a free model
infs app run openrouter/meta-llama/llama-3.1-8b-instruct --input '{"prompt":"What is 2+2?"}'

# Stream LLM response token by token
infs app run openrouter/openai/gpt-4o --input '{"prompt":"Count to 10"}' --stream

# Image generation via WaveSpeed AI
infs app run wavespeed/wavespeed-ai/flux-schnell --input '{"prompt":"a cat astronaut in space"}'

# Nano Banana 2 text-to-image via WaveSpeed AI
infs app run wavespeed/google/nano-banana-2 --input '{"prompt":"a serene mountain lake at sunset"}'

# Image generation via fal.ai
infs app run falai/fal-ai/flux/dev --input '{"prompt":"a cat astronaut in space"}'

# Save generated image to file (auto-detects extension)
infs app run wavespeed/google/nano-banana-2 --input '{"prompt":"a cat"}' --output image

# Save generated image with specific extension
infs app run wavespeed/google/nano-banana-2 --input '{"prompt":"a cat"}' --output image.png

# Use local image file with multimodal model (OpenRouter)
infs app run openrouter/openai/gpt-4o --file photo.jpg --prompt "What's in this image?"

# Use local image file with WaveSpeed image editing
infs app run wavespeed/google/nano-banana-2/edit --file input.png --prompt "Make it sepia"

# Multiple image files
infs app run openrouter/openai/gpt-4o --file img1.png --file img2.jpg --prompt "Compare these images"
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
**Status:** ✅ Fully implemented — live model listing and `app run` both work. Submits a job via `POST https://queue.fal.run/<app_id>` and polls for the result.

### Replicate

Replicate runs machine learning models in the cloud.

**Website:** https://replicate.com  
**Get an API key:** https://replicate.com/account/api-tokens  
**Status:** ✅ Fully implemented — live model listing and `app run` both work. Creates a prediction via the Replicate API and polls until complete.

### WaveSpeed AI

WaveSpeed AI provides fast image and video generation.

**Website:** https://wavespeed.ai  
**Get an API key:** https://wavespeed.ai/dashboard  
**Status:** ✅ Fully implemented — live model listing and inference both work.

**How it works:** `app run` submits a generation task via `POST /api/v3/<model_id>` and then polls `GET /api/v3/predictions/<task_id>` until the task completes (up to ~2 minutes). The generated image URLs are printed to stdout.

**Models included in built-in catalog:**
- `wavespeed/wavespeed-ai/flux-dev` — FLUX Dev
- `wavespeed/wavespeed-ai/flux-schnell` — FLUX Schnell
- `wavespeed/wavespeed-ai/wan2.1-t2v-480p` — Wan2.1 Text-to-Video 480p
- `wavespeed/google/nano-banana-2` — Google Nano Banana 2 (text-to-image)

Any model available on WaveSpeed can be run using its WaveSpeed model ID.

## Authentication

`infs` stores configuration in your OS's standard config directory:

- **Linux:** `~/.config/infs/`
- **macOS:** `~/Library/Application Support/infs/infs/`
- **Windows:** `%APPDATA%\infs\infs\`

Two files are used:
- `config.toml` — provider settings and metadata (non-sensitive)
- `credentials.toml` — API keys and secrets

> **Note:** On Unix, `credentials.toml` is written with file mode `0600` (owner read/write only). A future version will optionally integrate with the OS keychain via the `keyring` crate.

### Environment Variables (.env)

`infs` can automatically load provider credentials from environment variables. This is useful for:
- **CI/CD pipelines** — inject secrets via environment variables
- **Monorepos** — share a single `.env` file across multiple projects
- **Quick setup** — skip the interactive `provider connect` flow

#### How it works

1. Create a `.env` file in your project directory (or any parent directory up to 3 levels)
2. Set the environment variables for your providers
3. Run `infs` commands — credentials are automatically detected

#### Supported environment variables

| Provider | Environment Variable |
|----------|---------------------|
| OpenRouter | `OPENROUTER_API_KEY` |
| fal.ai | `FALAI_API_KEY` |
| Replicate | `REPLICATE_API_TOKEN` |
| WaveSpeed | `WAVESPEED_API_KEY` |

#### Example `.env` file

```bash
# .env
OPENROUTER_API_KEY=sk-or-v1-xxxxxxxxxxxxx
FALAI_API_KEY=xxxxxxxxxxxxx
REPLICATE_API_TOKEN=r8_xxxxxxxxxxxxx
WAVESPEED_API_KEY=xxxxxxxxxxxxx
```

#### Credentials priority

When multiple sources are available, `infs` uses this priority (highest wins):

1. **OS keychain** — most secure, managed via `infs provider connect`
2. **credentials.toml** — fallback file storage
3. **.env / environment variables** — lowest priority, good for defaults

#### Disabling .env loading

To skip `.env` loading and use only the credentials manager:

```bash
infs --no-env provider list
```

This is useful when you want to ensure only stored credentials are used, ignoring any environment variables.

> **Security note:** Never commit `.env` files to version control. Add `.env` to your `.gitignore`.

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

### Quick start with .env (recommended for development)

Create a `.env` file with your API keys:

```bash
# .env
OPENROUTER_API_KEY=sk-or-v1-xxxxxxxxxxxxx
```

Then run any command — no `provider connect` needed:

```bash
infs app list
infs app list openrouter
infs app run openrouter/meta-llama/llama-3.1-8b-instruct --input '{"prompt":"What is the capital of France?"}'
```

### Ask a question (interactive setup)

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

### Generate an image with WaveSpeed AI

```bash
infs provider connect wavespeed  # First time only — enter your API key
infs app run wavespeed/google/nano-banana-2 --input '{"prompt":"a serene mountain lake at sunset"}'
# Outputs one or more image URLs once generation completes
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
│   ├── falai.rs         # ✅ Full implementation (image, async queue)
│   ├── replicate.rs     # ✅ Full implementation (image, prediction polling)
│   └── wavespeed.rs     # ✅ Full implementation
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

### Running tests

```bash
cargo test
```

### Environment variables

| Variable | Purpose |
|---|---|
| `RUST_LOG` | Log level (`error`, `warn`, `info`, `debug`, `trace`) |

## Known Limitations

- Model listing requires an API key for fal.ai, Replicate, and WaveSpeed; a static fallback is shown when not connected

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full roadmap. Summary:

**Completed:**
- [x] fal.ai, Replicate, and WaveSpeed AI execution
- [x] OS keychain integration for credential storage (`keyring` crate)
- [x] `--json` output flag for machine-friendly output
- [x] Shell completion scripts (`infs completions bash/zsh/fish/powershell/elvish`)
- [x] Retry logic with exponential backoff
- [x] Self-update functionality (`infs self update`)
- [x] Streaming LLM responses (`--stream` flag)
- [x] Paginated model listing (`--page` and `--per-page` flags)
- [x] File output for image generation (`--output` flag)
- [x] File input support (`--file` flag for multimodal models)
- [ ] More providers (ElevenLabs, Stability AI, etc.)
- [ ] OAuth support for providers that require it

## License

MIT
