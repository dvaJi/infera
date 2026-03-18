# infs Roadmap

This document tracks the development roadmap for `infs`, a provider-agnostic CLI for running AI models from multiple providers through one consistent interface.

## Completed

- [x] **fal.ai execution** — Image generation via async queue API
- [x] **Replicate execution** — Image generation via prediction polling API
- [x] **WaveSpeed AI execution** — Image and video generation
- [x] **OS keychain integration** — Credentials stored securely in the OS keychain via `keyring` crate (falls back to `credentials.toml` when keychain is unavailable)
- [x] **`--json` output flag** — Machine-readable JSON output for scripting and automation (`infs --json ...`)
- [x] **Shell completion scripts** — Generate completions for bash, zsh, fish, PowerShell, and elvish (`infs completions <shell>`)
- [x] **Retry logic with exponential backoff** — Automatically retries transient network errors and HTTP 5xx responses with capped exponential backoff

## Planned

- [ ] **Streaming LLM responses** — Stream tokens as they are generated instead of waiting for the full response
- [ ] **Paginated model listing** — Handle providers with very large model catalogs via pagination
- [ ] **File output for image generation** — Automatically download and save generated images to a local file
- [ ] **File input support** — Pass local files (images, audio, etc.) as input to multimodal models
- [ ] **More providers** — ElevenLabs (audio), Stability AI (image), and others
- [ ] **OAuth support** — Support providers that use OAuth-based authentication flows

## Contributing

Have a feature suggestion or want to work on one of the planned items? See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute.
