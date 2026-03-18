# Running Apps

## Basic Usage

```bash
infs app run <provider/app-id> --input '<json>'
```

The `<provider/app-id>` format is `<provider-id>/<app-specific-id>`.
For example: `openrouter/anthropic/claude-sonnet-4-5`.

## Input Formats

### Inline JSON string

```bash
infs app run openrouter/openai/gpt-4o --input '{"prompt":"What is Rust?"}'
```

### JSON file

```bash
# Create input.json
echo '{"prompt": "Explain machine learning"}' > input.json

infs app run openrouter/openai/gpt-4o --input-file input.json
```

## LLM Examples (OpenRouter)

```bash
# Simple prompt
infs app run openrouter/anthropic/claude-sonnet-4-5 \
  --input '{"prompt":"Summarise the Rust ownership model"}'

# Structured messages (system + user)
infs app run openrouter/openai/gpt-4o --input '{
  "messages": [
    {"role": "system", "content": "You are a concise technical writer."},
    {"role": "user", "content": "Explain async/await in Rust"}
  ]
}'

# Free tier model
infs app run openrouter/meta-llama/llama-3.1-8b-instruct \
  --input '{"prompt":"What is 2 + 2?"}'

# Stream output token-by-token
infs app run openrouter/openai/gpt-4o \
  --input '{"prompt":"Write a haiku about Rust"}' --stream
```

## Image Generation Examples

```bash
# fal.ai — FLUX Dev
infs app run falai/fal-ai/flux/dev \
  --input '{"prompt":"a cat astronaut in space"}'

# WaveSpeed AI — FLUX Schnell (fast)
infs app run wavespeed/wavespeed-ai/flux-schnell \
  --input '{"prompt":"a serene mountain lake at sunset"}'

# WaveSpeed AI — save image locally
infs app run wavespeed/wavespeed-ai/flux-schnell \
  --input '{"prompt":"a serene mountain lake at sunset"}' \
  --output lake.png

# Replicate
infs app run replicate/stability-ai/sdxl \
  --input '{"prompt":"a futuristic city skyline"}'
```

## Video Generation Examples

```bash
# WaveSpeed AI text-to-video
infs app run wavespeed/wavespeed-ai/wan2.1-t2v-480p \
  --input '{"prompt":"a drone flying over snowy mountains"}'
```

## Saving Output

### Save image to file

Use `--output` to download and save generated images:

```bash
infs app run falai/fal-ai/flux/dev \
  --input '{"prompt":"a cat astronaut"}' \
  --output image.png
```

For multiple images, files are saved as `<stem>_0<ext>`, `<stem>_1<ext>`, etc.

### Machine-readable JSON output

```bash
infs --json app run openrouter/openai/gpt-4o --input '{"prompt":"Hello"}'
```

Example JSON response:

```json
{
  "output": {
    "Text": "Hello! How can I assist you today?"
  },
  "model": "openai/gpt-4o",
  "provider": "openrouter",
  "usage": {
    "prompt_tokens": 9,
    "completion_tokens": 10,
    "total_tokens": 19
  }
}
```

Image output response:

```json
{
  "output": {
    "ImageUrls": ["https://cdn.example.com/generated.png"]
  },
  "model": "fal-ai/flux/dev",
  "provider": "falai",
  "usage": null
}
```

## Multi-step Workflow Example

Chain LLM and image generation in a script:

```bash
#!/usr/bin/env bash
set -euo pipefail

# Step 1: Generate a creative image prompt using an LLM
PROMPT=$(infs --json app run openrouter/openai/gpt-4o \
  --input '{"prompt":"Write a vivid one-sentence image generation prompt for a surreal landscape"}' \
  | jq -r '.output.Text')

echo "Generated prompt: $PROMPT"

# Step 2: Generate the image
infs app run falai/fal-ai/flux/dev \
  --input "{\"prompt\": \"$PROMPT\"}" \
  --output surreal.png
```
