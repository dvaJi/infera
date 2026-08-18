# Authentication & Setup

## Install the CLI

Download the latest pre-built binary from [GitHub Releases](https://github.com/dvaJi/infera/releases/latest):

```bash
# Linux x86_64
curl -fsSL https://github.com/dvaJi/infera/releases/latest/download/infs-linux-x86_64 -o infs
chmod +x infs && sudo mv infs /usr/local/bin/

# Linux aarch64 (ARM)
curl -fsSL https://github.com/dvaJi/infera/releases/latest/download/infs-linux-aarch64 -o infs
chmod +x infs && sudo mv infs /usr/local/bin/

# macOS Apple Silicon
curl -fsSL https://github.com/dvaJi/infera/releases/latest/download/infs-macos-aarch64 -o infs
chmod +x infs && sudo mv infs /usr/local/bin/

# macOS Intel
curl -fsSL https://github.com/dvaJi/infera/releases/latest/download/infs-macos-x86_64 -o infs
chmod +x infs && sudo mv infs /usr/local/bin/

# Windows x86_64 — download infs-windows-x86_64.exe and add to PATH
```

Or install to a user-writable directory (no `sudo` required):

```bash
mkdir -p ~/.local/bin
curl -fsSL https://github.com/dvaJi/infera/releases/latest/download/infs-linux-x86_64 \
  -o ~/.local/bin/infs
chmod +x ~/.local/bin/infs
# Ensure ~/.local/bin is in your PATH
```

## Build from Source

Requires Rust 1.75+.  Install via [rustup](https://rustup.rs):

```bash
git clone https://github.com/dvaJi/infera && cd infera
cargo build --release
# Binary: ./target/release/infs
sudo mv target/release/infs /usr/local/bin/
# Or: cargo install --path .
```

## Connecting to Providers

Each provider uses API key authentication.  Run the interactive connect command:

```bash
infs provider connect <provider-id>
```

You will be prompted to enter your API key. The key is stored in the user-level `infs` JSON configuration at `config.json`; on Unix, the file is written with `0600` permissions.

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
