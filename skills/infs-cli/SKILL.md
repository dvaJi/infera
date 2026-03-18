---
name: infs-cli
description: >
  Run AI apps and models from multiple providers via the infs CLI.
  Supports LLMs (Claude, GPT-4o, Gemini, Llama, Mistral via OpenRouter),
  image generation (fal.ai, Replicate, WaveSpeed AI), and video generation.
  Use when running AI models, generating images or video, calling LLMs,
  discovering available apps, managing provider connections, or automating
  multi-step AI workflows.
  Triggers: infs, infera, ai model, run ai, openrouter, falai, fal.ai,
  replicate, wavespeed, image generation, video generation, llm, flux,
  claude api, gpt-4o, gemini, llama, provider connect.
allowed-tools: Bash(infs *) Bash(jq *)
---

# infs-cli

Run AI apps and models from multiple providers through one consistent CLI.
No provider-specific SDK required — one tool, one interface.

## Install the CLI

```bash
# Download the latest binary for your platform from GitHub Releases
# Linux x86_64
curl -fsSL https://github.com/dvaJi/infera/releases/latest/download/infs-linux-x86_64 -o infs
chmod +x infs && sudo mv infs /usr/local/bin/

# macOS Apple Silicon
curl -fsSL https://github.com/dvaJi/infera/releases/latest/download/infs-macos-aarch64 -o infs
chmod +x infs && sudo mv infs /usr/local/bin/

# macOS Intel
curl -fsSL https://github.com/dvaJi/infera/releases/latest/download/infs-macos-x86_64 -o infs
chmod +x infs && sudo mv infs /usr/local/bin/
```

Or build from source:

```bash
git clone https://github.com/dvaJi/infera && cd infera
cargo build --release
sudo mv target/release/infs /usr/local/bin/
```

## Connect a Provider

Each provider requires an API key.  Run the interactive connect command:

```bash
infs provider connect openrouter   # LLMs — get key at https://openrouter.ai/keys
infs provider connect falai        # image — get key at https://fal.ai/dashboard/keys
infs provider connect replicate    # image — get key at https://replicate.com/account/api-tokens
infs provider connect wavespeed    # image/video — get key at https://wavespeed.ai/dashboard
```

## Quick Examples

```bash
# Ask an LLM
infs app run openrouter/anthropic/claude-sonnet-4-5 \
  --input '{"prompt":"Explain quantum computing in one paragraph"}'

# Ask with structured messages
infs app run openrouter/openai/gpt-4o --input '{
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "What is Rust?"}
  ]
}'

# Stream LLM output token-by-token
infs app run openrouter/openai/gpt-4o \
  --input '{"prompt":"Write a haiku about Rust"}' --stream

# Generate an image and print URLs
infs app run falai/fal-ai/flux/dev \
  --input '{"prompt":"a cat astronaut in space"}'

# Generate an image and save it locally
infs app run wavespeed/wavespeed-ai/flux-schnell \
  --input '{"prompt":"a serene mountain lake at sunset"}' \
  --output lake.png

# List all available apps
infs app list

# Filter by category
infs app list --category image
infs app list --category llm

# Filter by provider
infs app list --provider openrouter
infs app list --provider falai

# Show details for an app
infs app show openrouter/anthropic/claude-sonnet-4-5

# List all providers and their connection status
infs provider list

# Show provider details
infs provider show openrouter

# Check health and diagnose connection issues
infs doctor
```

## Machine-readable Output

Add `--json` to any command to get structured JSON output, useful for scripting:

```bash
# JSON list of apps
infs --json app list --category llm

# JSON run response (includes output, model, provider, usage)
infs --json app run openrouter/openai/gpt-4o --input '{"prompt":"Hello"}'
```

## Commands

| Task | Command |
|---|---|
| List all providers | `infs provider list` |
| Connect to provider | `infs provider connect <id>` |
| Show provider details | `infs provider show <id>` |
| Disconnect from provider | `infs provider disconnect <id>` |
| List all apps | `infs app list` |
| Filter apps by category | `infs app list --category <image\|llm\|video\|audio>` |
| Filter apps by provider | `infs app list --provider <id>` |
| Paginate app list | `infs app list --page 2 --per-page 50` |
| Show app details | `infs app show <provider/app-id>` |
| Run an app | `infs app run <provider/app-id> --input '<json>'` |
| Run from JSON file | `infs app run <provider/app-id> --input-file input.json` |
| Run and stream output | `infs app run <provider/app-id> --input '<json>' --stream` |
| Run and save image | `infs app run <provider/app-id> --input '<json>' --output out.png` |
| JSON output | `infs --json <command>` |
| Show config path | `infs config path` |
| Health check | `infs doctor` |
| Shell completions | `infs completions bash\|zsh\|fish\|powershell\|elvish` |

## Supported Providers

| Provider ID | Category | Models |
|---|---|---|
| `openrouter` | LLM | Claude, GPT-4o, Gemini, Llama, Mistral, and hundreds more |
| `falai` | Image | FLUX, and many other fal.ai models |
| `replicate` | Image | Thousands of community models |
| `wavespeed` | Image / Video | FLUX Schnell, FLUX Dev, Wan2.1, and more |

## Reference Files

- [Authentication & Setup](./references/authentication.md)
- [Discovering Apps](./references/app-discovery.md)
- [Running Apps](./references/running-apps.md)
- [CLI Reference](./references/cli-reference.md)
