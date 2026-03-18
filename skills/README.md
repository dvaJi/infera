# infs Agent Skills

Agent skills for the [`infs`](https://github.com/dvaJi/infera) CLI — a fast, provider-agnostic interface for AI apps and models.

Install these skills in your agent environment to enable running LLMs, generating images, and automating AI workflows using the `infs` CLI.

## Available Skills

| Skill | Description | Install |
|---|---|---|
| [`infs-cli`](./infs-cli/SKILL.md) | Full CLI wrapper — providers, apps, image & LLM runs | `npx skills add dvaJi/infera/skills@infs-cli` |
| [`infs-llm`](./infs-llm/SKILL.md) | Run LLMs via OpenRouter (Claude, GPT-4o, Gemini, Llama, Mistral, …) | `npx skills add dvaJi/infera/skills@infs-llm` |
| [`infs-image`](./infs-image/SKILL.md) | Generate images via fal.ai, Replicate, and WaveSpeed AI | `npx skills add dvaJi/infera/skills@infs-image` |

## Quick Start

### 1. Install the `infs` CLI

```bash
# Download the latest binary for your platform from GitHub Releases
# e.g. on Linux x86_64:
curl -fsSL https://github.com/dvaJi/infera/releases/latest/download/infs-linux-x86_64 -o infs
chmod +x infs
sudo mv infs /usr/local/bin/
# Or without sudo:
# mkdir -p ~/.local/bin && mv infs ~/.local/bin/
```

### 2. Connect to a provider

```bash
infs provider connect openrouter   # LLMs
infs provider connect falai        # image generation
infs provider connect wavespeed    # image / video generation
infs provider connect replicate    # image generation
```

### 3. Run an AI app

```bash
# Ask an LLM
infs app run openrouter/anthropic/claude-sonnet-4-5 --input '{"prompt":"Explain quantum computing"}'

# Generate an image
infs app run falai/fal-ai/flux/dev --input '{"prompt":"a cat astronaut in space"}'
```

## Composing Skills in Agent Workflows

Skills are designed to be composable. Use `infs --json` output to pipe results between steps:

```bash
# Step 1: ask an LLM to describe an image
DESCRIPTION=$(infs --json app run openrouter/openai/gpt-4o \
  --input '{"prompt":"Describe a surreal landscape for an image generation prompt"}' \
  | jq -r '.output.Text')

# Step 2: generate the image
infs app run falai/fal-ai/flux/dev --input "{\"prompt\": \"$DESCRIPTION\"}"
```

## Reference

- [CLI Reference](./infs-cli/references/cli-reference.md)
- [Authentication & Setup](./infs-cli/references/authentication.md)
- [Discovering Apps](./infs-cli/references/app-discovery.md)
- [Running Apps](./infs-cli/references/running-apps.md)
