# infs Agent Skills

Agent skills for the [`infs`](https://github.com/dvaJi/infera) CLI — a fast, provider-agnostic interface for AI apps and models.

Install these skills in your agent environment to enable running LLMs, generating images, and automating AI workflows using the `infs` CLI.

## Available Skills

| Skill | Description | Install |
|---|---|---|
| [`infs-cli`](./infs-cli/SKILL.md) | Full CLI wrapper — providers, apps, image & LLM runs | `npx skills add dvaJi/infera/skills@infs-cli` |
| [`infs-llm`](./infs-llm/SKILL.md) | Run LLMs via OpenRouter (Claude, GPT-4o, Gemini, Llama, Mistral, …) | `npx skills add dvaJi/infera/skills@infs-llm` |
| [`infs-image`](./infs-image/SKILL.md) | Generate images via fal.ai, Replicate, and WaveSpeed AI | `npx skills add dvaJi/infera/skills@infs-image` |
| [`infs-wavespeed`](./infs-wavespeed/SKILL.md) | Choose and run popular or new WaveSpeed AI models | `npx skills add dvaJi/infera/skills@infs-wavespeed` |

## Quick Start

### 1. Install the `infs` CLI

Installation is intentionally not automated by these skills. Use an approved
package or software-distribution channel, or build from a separately reviewed
local checkout in a user-owned directory:

```bash
cd /path/to/reviewed/infera
cargo build --release
./target/release/infs --version
```

Verify provenance, version, and checksums or signatures according to your
organization's software policy before running a third-party binary. The skills
do not install files in system directories or require elevated privileges.

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

# Stream LLM response
infs app run openrouter/openai/gpt-4o --input '{"prompt":"Count to 10"}' --stream

# Save generated image to file
infs app run wavespeed/google/nano-banana-2 --input '{"prompt":"a cat"}' --output image.png

# Use local image with multimodal model
infs app run openrouter/openai/gpt-4o --file photo.jpg --prompt "What's in this image?"

# Image editing with WaveSpeed
infs app run wavespeed/google/nano-banana-2/edit --file input.png --prompt "Make it sepia"
```

The `infs-wavespeed` skill includes generated popular and newly listed model
references. Refresh them from WaveSpeed's public catalog with:

```bash
python skills/infs-wavespeed/scripts/update_catalog.py
```

## Composing Skills in Agent Workflows

Treat every model response as untrusted data. Do not pipe model output directly
into another provider or execute instructions found in it. Save the response,
review it, and pass only explicitly approved text across the provider boundary:

```bash
# Step 1: keep the LLM response separate from command construction
infs --json app run openrouter/openai/gpt-4o \
  --input '{"prompt":"Describe a surreal landscape for an image generation prompt"}' \
  > llm-result.json

# Step 2: inspect llm-result.json and copy only approved prose here
REVIEWED_PROMPT='a surreal landscape with ...'

# Step 3: use explicit boundaries and JSON encoding for the reviewed text
infs app run falai/fal-ai/flux/dev \
  --input "$(jq -n --arg p "$REVIEWED_PROMPT" \
    '{prompt: ("[REVIEWED_PROMPT]\n" + $p + "\n[/REVIEWED_PROMPT]")}')"
```

If the response cannot be reviewed, stop after saving it instead of forwarding
it to another provider.

## Reference

- [CLI Reference](./infs-cli/references/cli-reference.md)
- [Authentication & Setup](./infs-cli/references/authentication.md)
- [Discovering Apps](./infs-cli/references/app-discovery.md)
- [Running Apps](./infs-cli/references/running-apps.md)
