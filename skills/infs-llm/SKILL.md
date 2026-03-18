---
name: infs-llm
description: >
  Run large language models (LLMs) via the infs CLI using OpenRouter.
  Access Claude (Anthropic), GPT-4o (OpenAI), Gemini (Google), Llama (Meta),
  Mistral, and hundreds of other models through one consistent interface.
  Use when querying an LLM, generating text, summarising content, code
  generation, Q&A, chat with system prompts, or streaming completions.
  Triggers: llm, openrouter, claude, gpt-4o, gemini, llama, mistral,
  language model, text generation, chat completion, ai assistant, summarise,
  code generation, explain code, stream llm.
allowed-tools: Bash(infs *) Bash(jq *)
---

# infs-llm

Run LLMs from OpenRouter using the `infs` CLI.
Access Claude, GPT-4o, Gemini, Llama, Mistral, and hundreds more — all with
the same command.

## Prerequisites

Connect to OpenRouter (one-time setup):

```bash
infs provider connect openrouter
# Enter your API key from https://openrouter.ai/keys
```

## Quick Examples

```bash
# Ask a question
infs app run openrouter/anthropic/claude-sonnet-4-5 \
  --input '{"prompt":"Explain the Rust ownership model"}'

# GPT-4o
infs app run openrouter/openai/gpt-4o \
  --input '{"prompt":"Write a haiku about Rust"}'

# Free model (Llama 3.1 8B)
infs app run openrouter/meta-llama/llama-3.1-8b-instruct \
  --input '{"prompt":"What is 2 + 2?"}'

# Stream output token by token
infs app run openrouter/openai/gpt-4o \
  --input '{"prompt":"Write a short story about a robot"}' --stream

# System + user message format
infs app run openrouter/openai/gpt-4o --input '{
  "messages": [
    {"role": "system", "content": "You are a concise technical writer."},
    {"role": "user", "content": "Explain async/await in Rust in 3 sentences."}
  ]
}'

# Input from file
echo '{"prompt":"Summarise the README for me"}' > prompt.json
infs app run openrouter/anthropic/claude-sonnet-4-5 --input-file prompt.json
```

## Available Models

List all LLM models:

```bash
infs app list --category llm
```

Common models:

| App ID | Model | Notes |
|---|---|---|
| `openrouter/openai/gpt-4o` | GPT-4o | OpenAI flagship |
| `openrouter/openai/gpt-4o-mini` | GPT-4o Mini | Faster, cheaper |
| `openrouter/anthropic/claude-sonnet-4-5` | Claude Sonnet 4.5 | Anthropic |
| `openrouter/google/gemini-flash-1.5` | Gemini Flash 1.5 | Google |
| `openrouter/meta-llama/llama-3.1-8b-instruct` | Llama 3.1 8B | Free |
| `openrouter/mistralai/mistral-7b-instruct` | Mistral 7B | Free |

Any model available on [OpenRouter](https://openrouter.ai/models) can be run
by using its full model ID prefixed with `openrouter/`.

## Machine-readable JSON Output

```bash
infs --json app run openrouter/openai/gpt-4o --input '{"prompt":"Hello"}'
```

Response format:

```json
{
  "output": {
    "type": "Text",
    "data": "Hello! How can I help you today?"
  },
  "model": "openai/gpt-4o",
  "provider": "openrouter",
  "usage": {
    "prompt_tokens": 9,
    "completion_tokens": 12,
    "total_tokens": 21
  }
}
```

Use `jq` to extract the text from `.output.data`:

```bash
infs --json app run openrouter/openai/gpt-4o \
  --input '{"prompt":"What year is it?"}' \
  | jq -r '.output.data'
```

## Scripting with LLMs

```bash
#!/usr/bin/env bash
# Summarise a file using Claude
# Use jq to safely build JSON input so the file content is correctly escaped
FILE="$1"
infs app run openrouter/anthropic/claude-sonnet-4-5 \
  --input "$(jq -n --arg content "$(cat "$FILE")" \
    '{prompt: ("Summarise the following text:\n\n" + $content)}')"
```
