# Changelog

## [0.5.0](https://github.com/dvaJi/infera/compare/infs-v0.4.0...infs-v0.5.0) (2026-04-21)


### Features

* add agent skills section to README with installation instructions and available skills ([6b4b0b8](https://github.com/dvaJi/infera/commit/6b4b0b87f66b49d8563995156d5df22c54cb04c4))


### Bug Fixes

* address code review feedback on PR [#29](https://github.com/dvaJi/infera/issues/29) ([7bce7e5](https://github.com/dvaJi/infera/commit/7bce7e58c91970c2bedabdf2c9138532feee664c))
* address critical review comments - fix input/file conflict and multimodal deserialization ([ccd900e](https://github.com/dvaJi/infera/commit/ccd900e0258b790f94fd87f997ac69089cfb1eed))
* address review feedback on keychain warning and polling comment ([b1448e4](https://github.com/dvaJi/infera/commit/b1448e4c83851e4a3dfe9c34be0eecf33c7e3e91))
* CLI improvements for v1 release - fix pagination and file flag bugs ([82712ec](https://github.com/dvaJi/infera/commit/82712ecd70ee78094c8333ceb3d28c6af117c641))
* correct credential priority order and add source detection ([f20d1c3](https://github.com/dvaJi/infera/commit/f20d1c3f42027e7f3f2268437b0a981014b16a14))
* correct credential priority order and add source detection ([3c65557](https://github.com/dvaJi/infera/commit/3c6555772090e684a09d22941b19134c6407ae29))
* handle WaveSpeed auth failures gracefully and correct poll URL ([b295f5c](https://github.com/dvaJi/infera/commit/b295f5c22b7b88897ca0c7c2e10e79b80b7caab5))
* handle WaveSpeed auth failures gracefully and correct poll URL ([683faef](https://github.com/dvaJi/infera/commit/683faef253d0227e93a5106462b5d314d65e7d7b))
* keychain credentials should always override file credentials ([1818cf4](https://github.com/dvaJi/infera/commit/1818cf4043150f6f5d0e7d5ee292b2a129721001))
* prevent env-sourced credentials from other providers being persisted to disk ([7e0fe64](https://github.com/dvaJi/infera/commit/7e0fe6490b4c5bfd873d3deae6270e48420733fe))
* refactor pagination architecture to fix app discovery bug ([637ce15](https://github.com/dvaJi/infera/commit/637ce15c0d91658114de09eea1a305bab84af0f8))

## [0.4.0](https://github.com/dvaJi/infera/compare/infs-v0.3.0...infs-v0.4.0) (2026-04-05)


### Features

* add .env file support for provider credentials ([8c02842](https://github.com/dvaJi/infera/commit/8c02842a82af2078067898c70a216cf15a8d59a0))
* add .env file support for provider credentials ([8fdc998](https://github.com/dvaJi/infera/commit/8fdc99814bd9cd476b66a57e1768440039998afb))
* add Chocolatey package support with installation and uninstallation scripts ([90cdd83](https://github.com/dvaJi/infera/commit/90cdd830dacf656a4b1676d26ebfaea4bf58531e))
* add support for file input and output in multimodal models, including streaming responses and pagination ([e3f46f9](https://github.com/dvaJi/infera/commit/e3f46f96b75054527502da22070949d273dc5909))
* implement pagination support for app listing across providers ([b41d5dd](https://github.com/dvaJi/infera/commit/b41d5dd8cbfd1405ebb7671e305baa55d671e3d4))
* improve app list provider discovery ([bdb5f69](https://github.com/dvaJi/infera/commit/bdb5f69ca62e4bb4b9a8c6bfc1f03da85a075ec5))
* refactor run_app function to use RunAppArgs struct for improved argument handling ([98335e5](https://github.com/dvaJi/infera/commit/98335e5109d9a412cefcad2e746639305f55a5c5))


### Bug Fixes

* disable environment loading in load_config function ([b838f78](https://github.com/dvaJi/infera/commit/b838f7827a91987b6645fe5d435204c1506ee2e8))
* handle infs-v prefix in version tag parsing ([930cd7a](https://github.com/dvaJi/infera/commit/930cd7a31eba08e4d5bb288a98d9fd2fdfdc5092))
* honor --no-env and stabilize dotenv tests ([ef0249e](https://github.com/dvaJi/infera/commit/ef0249e4dc1ef0134af18206178d03b2227c930e))


### Documentation

* add .env file support and update documentation ([1ea0f25](https://github.com/dvaJi/infera/commit/1ea0f25ac12eca02fdadc3dea5db8c3f2ba89b05))
* add comprehensive .env usage documentation ([70ee1bf](https://github.com/dvaJi/infera/commit/70ee1bf419b097f4f3fb94b0db13230489fd9151))
* require README updates for user-facing changes ([90618cf](https://github.com/dvaJi/infera/commit/90618cf0ebb94ae0ffe88acfeb3480195f8058ac))

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
