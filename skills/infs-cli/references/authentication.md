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
print the config file or enable shell tracing while credentials are in
the environment. For automation, inject secrets from your CI or workstation
secret manager into the process environment and remove them from logs.

The CLI uses the user-level `config.json`, environment variables, and
runtime-only credentials from supported provider CLIs. Keep `.env` files
ignored by version control and use them only from trusted project directories.
Rotate a key immediately if it is exposed.

## Connecting to Providers

Each provider uses API key authentication.  Run the interactive connect command:

```bash
infs provider connect <provider-id>
```

You will be prompted to enter your API key. The key is stored in the user-level `infs` JSON configuration at `config.json`; on Unix, the file is written with `0600` permissions.

When upgrading from an earlier infs release, existing provider settings and credentials are imported into `config.json` on first load. The previous files and keychain entries are left untouched.

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

`connect` validates the API key with the provider before saving it. For offline setup, use `infs provider connect <provider-id> --skip-validation`.

## Reuse provider CLI credentials

`infs` reads credentials saved by the official provider CLIs when no credential is already stored in `infs` configuration:

- WaveSpeed: the key saved by `wavespeed login`
- Replicate: the token saved by `replicate auth login`

These files are read without modifying them.

## Verify Connection

```bash
# List all providers and their connection status
infs provider list

# Run the health check
infs doctor
```

To inspect one provider without making a network request:

```bash
infs provider status openrouter
infs --json provider status openrouter
```

## Disconnect

```bash
infs provider disconnect openrouter
```

Disconnecting removes stored credentials and disables environment/provider-CLI fallbacks for that provider until the next `connect`.

## Config File Location

```bash
infs config path
```

Default locations:

| OS | Path |
|---|---|
| Linux | `~/.config/infs/config.json` |
| macOS | `~/Library/Application Support/infs/config.json` |
| Windows | `%APPDATA%\infs\config.json` |

The JSON file contains provider settings and API keys. Use `infs config path` for the exact path on the current machine.
