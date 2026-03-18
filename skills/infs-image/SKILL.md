---
name: infs-image
description: >
  Generate images via the infs CLI using fal.ai, Replicate, or WaveSpeed AI.
  Supports text-to-image generation with models including FLUX Dev, FLUX Schnell,
  Stable Diffusion XL, and many more. Use when generating images from text prompts,
  saving generated images locally, or automating image creation in scripts.
  Triggers: image generation, text to image, generate image, flux, stable diffusion,
  sdxl, fal.ai, falai, replicate, wavespeed, ai art, diffusion model, create image,
  render image, image synthesis.
allowed-tools: Bash(infs *)
---

# infs-image

Generate images from text prompts using the `infs` CLI.
Supports multiple providers: fal.ai, Replicate, and WaveSpeed AI.

## Prerequisites

Connect to at least one image provider (one-time setup):

```bash
# fal.ai — get key at https://fal.ai/dashboard/keys
infs provider connect falai

# WaveSpeed AI — get key at https://wavespeed.ai/dashboard
infs provider connect wavespeed

# Replicate — get key at https://replicate.com/account/api-tokens
infs provider connect replicate
```

## Quick Examples

```bash
# fal.ai — FLUX Dev (high quality)
infs app run falai/fal-ai/flux/dev \
  --input '{"prompt":"a cat astronaut in space, photorealistic"}'

# WaveSpeed AI — FLUX Schnell (fast)
infs app run wavespeed/wavespeed-ai/flux-schnell \
  --input '{"prompt":"a serene mountain lake at sunset"}'

# WaveSpeed AI — FLUX Dev
infs app run wavespeed/wavespeed-ai/flux-dev \
  --input '{"prompt":"a cyberpunk city at night, neon lights"}'

# Save image to a local file
infs app run falai/fal-ai/flux/dev \
  --input '{"prompt":"a waterfall in a tropical forest"}' \
  --output waterfall.png

# Generate multiple images (if supported by the model)
infs app run wavespeed/wavespeed-ai/flux-schnell \
  --input '{"prompt":"abstract geometric art", "num_images": 4}' \
  --output art.png
# Saved as art_0.png, art_1.png, art_2.png, art_3.png
```

## Available Image Models

```bash
infs app list --category image
```

Common models:

| App ID | Provider | Notes |
|---|---|---|
| `falai/fal-ai/flux/dev` | fal.ai | FLUX Dev — high quality |
| `wavespeed/wavespeed-ai/flux-dev` | WaveSpeed AI | FLUX Dev |
| `wavespeed/wavespeed-ai/flux-schnell` | WaveSpeed AI | FLUX Schnell — fast |
| `wavespeed/google/nano-banana-2` | WaveSpeed AI | Google Nano Banana 2 |

Discover more models with `infs app list --category image`.

## Output Formats

By default, image URLs are printed to stdout:

```bash
infs app run wavespeed/wavespeed-ai/flux-schnell \
  --input '{"prompt":"a red balloon"}'
# https://cdn.wavespeed.ai/results/abc123.png
```

Use `--output` to download and save images locally:

```bash
infs app run wavespeed/wavespeed-ai/flux-schnell \
  --input '{"prompt":"a red balloon"}' \
  --output balloon.png
# Saved image to: balloon.png
```

## Machine-readable JSON Output

```bash
infs --json app run falai/fal-ai/flux/dev \
  --input '{"prompt":"a red balloon"}'
```

Response format:

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

Extract URLs with `jq`:

```bash
infs --json app run wavespeed/wavespeed-ai/flux-schnell \
  --input '{"prompt":"a red balloon"}' \
  | jq -r '.output.ImageUrls[]'
```

## Scripting Examples

### Download and display multiple images

```bash
#!/usr/bin/env bash
PROMPTS=(
  "a red apple on a wooden table"
  "a blue ocean wave at sunset"
  "a green forest with morning mist"
)

for PROMPT in "${PROMPTS[@]}"; do
  SAFE_NAME=$(echo "$PROMPT" | tr ' ' '_' | cut -c1-30)
  infs app run wavespeed/wavespeed-ai/flux-schnell \
    --input "{\"prompt\": \"$PROMPT\"}" \
    --output "${SAFE_NAME}.png"
done
```

### LLM-to-image pipeline

```bash
#!/usr/bin/env bash
# Generate a creative prompt with an LLM, then create the image

PROMPT=$(infs --json app run openrouter/openai/gpt-4o \
  --input '{"prompt":"Write a vivid one-sentence image generation prompt for a surreal landscape"}' \
  | jq -r '.output.Text')

echo "Using prompt: $PROMPT"

infs app run falai/fal-ai/flux/dev \
  --input "{\"prompt\": \"$PROMPT\"}" \
  --output surreal_landscape.png
```
