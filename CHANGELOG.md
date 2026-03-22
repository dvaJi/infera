# Changelog

## [0.3.0](https://github.com/dvaJi/infera/compare/infs-v0.2.0...infs-v0.3.0) (2026-03-22)


### Features

* add self-update functionality and install scripts ([e86da6a](https://github.com/dvaJi/infera/commit/e86da6a4f97a01ab4ffad37f3be1ff712441bfc0))
* add self-update functionality and install scripts ([59ac56d](https://github.com/dvaJi/infera/commit/59ac56d02e78459d24b7826e3f5a5169e888cbeb))


### Bug Fixes

* address CI and review feedback ([1009190](https://github.com/dvaJi/infera/commit/10091904394f73d6a5c40609dcfe71ee4d224605))
* address CI and review feedback ([62d93bf](https://github.com/dvaJi/infera/commit/62d93bfd273d0cc9e82d4fca1e043f6acedfb6b6))
* address review feedback ([1b88195](https://github.com/dvaJi/infera/commit/1b88195865913a9c9605acf7e3b56a19fdf340a2))
* **ci:** drop Intel macOS build (requires paid runner) ([f0bae15](https://github.com/dvaJi/infera/commit/f0bae15eab51e9e4fe981c131a31ff177141cb80))
* **ci:** update macOS runners and fix release asset upload ([ce7e5b4](https://github.com/dvaJi/infera/commit/ce7e5b4b1844fba52d6c7cf760be94dfaff1406e))
* streamline release workflow by removing push and workflow_run triggers ([8ef553f](https://github.com/dvaJi/infera/commit/8ef553faf916d76ecf0c099cc8ab30c067572541))
* update actions/checkout and actions/cache versions in CI and release workflows ([821e17e](https://github.com/dvaJi/infera/commit/821e17ec3e53a1541c2481f3c351cadf7b0552cb))
* update release workflow to trigger on infs-v*.*.* tags ([645b6fb](https://github.com/dvaJi/infera/commit/645b6fba13119bd8eaeb63ab9233650cd08faa9a))

## [0.2.0](https://github.com/dvaJi/infera/compare/infs-v0.1.0...infs-v0.2.0) (2026-03-18)


### Features

* Add `infs` — provider-agnostic Rust CLI for AI model execution ([deb01bf](https://github.com/dvaJi/infera/commit/deb01bfd61ad91fc5cbb38457db9eabfcc0fa336))
* add advanced automation and machine-friendly CLI features ([a0ae487](https://github.com/dvaJi/infera/commit/a0ae4871122530d6b0e84e072b093e394aa34ae2))
* add agent skills for the infs CLI (skills.sh integration) ([71492b8](https://github.com/dvaJi/infera/commit/71492b88edd0da27b5ad0c3a26b616f5ddc4eeb5))
* add agent skills for the infs CLI (skills.sh integration) ([6a3a8c7](https://github.com/dvaJi/infera/commit/6a3a8c7c9b62ea3e110d0844c809df2ec4a00715))
* Add infs Rust CLI with provider registry, OpenRouter adapter, and scaffolded image providers ([c44d6aa](https://github.com/dvaJi/infera/commit/c44d6aad183d133b1ac3a45b9b25445957812acd))
* Fetch models live from provider APIs for app list command ([2824f0c](https://github.com/dvaJi/infera/commit/2824f0cc245450cfa0a4ddd9bfbc47384212a7fa))
* initial Rust CLI project for infs ([7c7846d](https://github.com/dvaJi/infera/commit/7c7846ddd8f266a2af60b0bcc8f50fb43313655f))
* integrate OS keychain via keyring crate for credential storage ([1b1c68b](https://github.com/dvaJi/infera/commit/1b1c68b0c97ed905816faf44ed67b89d44237b3d))
* integrate OS keychain via keyring crate for credential storage ([21b303c](https://github.com/dvaJi/infera/commit/21b303ce93cbaa4a2a1247a87ec28ab57467b740))


### Bug Fixes

* address keyring review comments — sort keys, ignore e2e test, clear stale metadata, delete orphaned entries ([362d38a](https://github.com/dvaJi/infera/commit/362d38a7c118f294f11332c28566ba7e7ace00e5))
* address PR review feedback on retry, streaming, pagination, and image download ([620deec](https://github.com/dvaJi/infera/commit/620deecf4188b5e14c0e24dfa79252798b987757))
* Apply all code review feedback from automated PR review ([e3b567e](https://github.com/dvaJi/infera/commit/e3b567e6b3c06a020f97ec4a91af58e750a80873))


### Documentation

* apply review feedback on documentation and release workflow ([d5d35ce](https://github.com/dvaJi/infera/commit/d5d35ce728da3e353c4a10749a0cb0579aea6cb1))
* fix JSON output format, allowed-tools, and unsafe interpolation in skills ([f37c963](https://github.com/dvaJi/infera/commit/f37c9636e0aca6ee4161a175f5060df71c1f82b8))
* update README and add CONTRIBUTING.md and AGENTS.md ([d26aec3](https://github.com/dvaJi/infera/commit/d26aec395819092fcf9ac4210a305848b6798362))
* update roadmap to reflect completed features and add ROADMAP.md ([3802423](https://github.com/dvaJi/infera/commit/3802423488b412aae15ca4971437d9074e344458))
* update roadmap to reflect completed features and add ROADMAP.md ([915fe23](https://github.com/dvaJi/infera/commit/915fe236e056ca28622435a03c375f92e7383159))
