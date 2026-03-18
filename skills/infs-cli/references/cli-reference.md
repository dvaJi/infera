# CLI Reference

## Global Flags

| Flag | Description |
|---|---|
| `--json` | Output machine-readable JSON instead of formatted text |
| `--help`, `-h` | Show help |
| `--version`, `-V` | Show version |

## Provider Commands

### `infs provider list`

List all supported providers and their connection status.

```bash
infs provider list
infs --json provider list
```

### `infs provider connect <id>`

Connect to a provider interactively (prompts for API key).

```bash
infs provider connect openrouter
infs provider connect falai
infs provider connect replicate
infs provider connect wavespeed
```

### `infs provider show <id>`

Show provider details: description, status, categories, auth methods, and available apps.

```bash
infs provider show openrouter
infs --json provider show falai
```

### `infs provider disconnect <id>`

Remove stored credentials for a provider.

```bash
infs provider disconnect openrouter
```

## App Commands

### `infs app list`

List available apps across all providers.

```bash
infs app list

# Flags
--category <image|llm|video|audio|other>   # Filter by category
--provider <id>                             # Filter by provider
--page <n>                                  # Page number (default: 1)
--per-page <n>                              # Results per page (default: 20)
```

Examples:

```bash
infs app list --category llm
infs app list --provider openrouter --per-page 50
infs --json app list --category image
```

### `infs app show <provider/app-id>`

Show details for a specific app.

```bash
infs app show openrouter/anthropic/claude-sonnet-4-5
infs --json app show falai/fal-ai/flux/dev
```

### `infs app run <provider/app-id>`

Run an app with JSON input.

```bash
infs app run <provider/app-id> --input '<json>'
infs app run <provider/app-id> --input-file <path>

# Flags
--input, -i <json>       Inline JSON input string
--input-file <path>      Read JSON input from a file (alternative to --input)
--stream                 Stream output token by token (LLM providers only)
--output, -o <path>      Save image output to this file path
```

Examples:

```bash
# Inline JSON
infs app run openrouter/openai/gpt-4o --input '{"prompt":"Hello"}'

# From file
infs app run openrouter/openai/gpt-4o --input-file prompt.json

# Stream
infs app run openrouter/openai/gpt-4o \
  --input '{"prompt":"Write a poem"}' --stream

# Save image
infs app run wavespeed/wavespeed-ai/flux-schnell \
  --input '{"prompt":"a mountain"}' --output mountain.png

# JSON output
infs --json app run openrouter/openai/gpt-4o --input '{"prompt":"Hi"}'
```

## Config Commands

### `infs config path`

Print the path to the configuration directory.

```bash
infs config path
```

## Utilities

### `infs doctor`

Check system health: verify provider connections and diagnose issues.

```bash
infs doctor
```

### `infs completions <shell>`

Generate shell completion scripts.

```bash
infs completions bash    >> ~/.bash_completion.d/infs
infs completions zsh     >> ~/.zsh/completions/_infs
infs completions fish    > ~/.config/fish/completions/infs.fish
infs completions powershell
infs completions elvish
```

## App ID Format

All app IDs follow the `<provider-id>/<app-specific-id>` format:

| Example | Provider | App |
|---|---|---|
| `openrouter/anthropic/claude-sonnet-4-5` | `openrouter` | `anthropic/claude-sonnet-4-5` |
| `openrouter/openai/gpt-4o` | `openrouter` | `openai/gpt-4o` |
| `falai/fal-ai/flux/dev` | `falai` | `fal-ai/flux/dev` |
| `replicate/stability-ai/sdxl` | `replicate` | `stability-ai/sdxl` |
| `wavespeed/wavespeed-ai/flux-schnell` | `wavespeed` | `wavespeed-ai/flux-schnell` |

## Environment Variables

| Variable | Purpose |
|---|---|
| `RUST_LOG` | Log level: `error`, `warn`, `info`, `debug`, `trace` |

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | Error (details printed to stderr) |
