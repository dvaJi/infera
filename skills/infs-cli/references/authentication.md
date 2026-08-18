# Authentication & Setup

## Install the CLI

Installation is intentionally not automated by this skill. Obtain `infs`
through an approved package or software-distribution channel, or build it from
a separately reviewed local checkout. For a source build, use a user-owned
working directory:

```bash
cd /path/to/reviewed/infera
cargo build --release
./target/release/infs --version
```

Verify the source, version, and any available checksums or signatures using
your organization's software policy before running the result. Do not execute
remote installer scripts or place an unreviewed binary in a system directory.

## Credential security

Use the interactive connect command so the API key is entered at a masked
prompt:

```bash
infs provider connect <provider-id>
```

Never pass a key as a command-line argument, put it in JSON input or prompts,
commit it to a repository, or include it in logs or agent messages. Do not
print the credentials file or enable shell tracing while credentials are in
the environment. For automation, inject secrets from your CI or workstation
secret manager into the process environment and remove them from logs.

The CLI can use the OS keychain, environment variables, or the protected file
fallback. Keep `.env` files ignored by version control and use them only from
trusted project directories. Rotate a key immediately if it is exposed.

## Connecting to Providers

Each provider uses API key authentication.  Run the interactive connect command:

```bash
infs provider connect <provider-id>
```

You will be prompted to enter your API key.  The key is stored securely — in the OS keychain when available, or in `credentials.toml` with `0600` permissions on Unix.

### Provider IDs and Key URLs

| Provider | ID | Get API Key |
|---|---|---|
| OpenRouter | `openrouter` | https://openrouter.ai/keys |
| fal.ai | `falai` | https://fal.ai/dashboard/keys |
| Replicate | `replicate` | https://replicate.com/account/api-tokens |
| WaveSpeed AI | `wavespeed` | https://wavespeed.ai/dashboard |

### Connect Examples

```bash
infs provider connect openrouter
infs provider connect falai
infs provider connect replicate
infs provider connect wavespeed
```

## Verify Connection

```bash
# List all providers and their connection status
infs provider list

# Run the health check
infs doctor
```

## Disconnect

```bash
infs provider disconnect openrouter
```

## Config File Location

```bash
infs config path
```

Default locations:

| OS | Path |
|---|---|
| Linux | `~/.config/infs/` |
| macOS | `~/Library/Application Support/infs/infs/` |
| Windows | `%APPDATA%\infs\infs\` |

Two files are used:

- `config.toml` — provider settings (non-sensitive)
- `credentials.toml` — API keys (sensitive, mode `0600` on Unix)
