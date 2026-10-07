# Changelog

## Unreleased

- Expose native Responses creation and streaming through the configured OpenAI delegate.
- Upgrade to async-openai 0.42.1 with the responses feature.

All notable changes to `floopy-sdk` (Rust) are documented in this file. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
the project adheres to [Semantic Versioning](https://semver.org/).

Releases are produced by `release-please` from Conventional Commits.

## [0.5.0](https://github.com/FloopyAI/floopy-rust/compare/floopy-sdk-v0.4.0...floopy-sdk-v0.5.0) (2026-10-07)


### Added

* expose native Responses API and upgrade async-openai ([8c48773](https://github.com/FloopyAI/floopy-rust/commit/8c4877393234bbea044ff2e9c9f907fa819d6ecd))
* expose native Responses API and upgrade async-openai ([feda0c0](https://github.com/FloopyAI/floopy-rust/commit/feda0c021c798db184d1f2ef47f78e86f192507b))

## [0.4.0](https://github.com/FloopyAI/floopy-rust/compare/floopy-sdk-v0.3.0...floopy-sdk-v0.4.0) (2026-07-06)


### Added

* add max_completion_tokens to routing explain params ([a2abfef](https://github.com/FloopyAI/floopy-rust/commit/a2abfef2a50d3086e351dd88f405624c8ebc0da9))
* add max_completion_tokens to routing explain params ([c65b5be](https://github.com/FloopyAI/floopy-rust/commit/c65b5be3030096eb494fd8310302b627e2f52185))

## [0.3.0](https://github.com/FloopyAI/floopy-rust/compare/floopy-sdk-v0.2.1...floopy-sdk-v0.3.0) (2026-05-19)


### Added

* add Batch and Files API resources ([b82dbf5](https://github.com/FloopyAI/floopy-rust/commit/b82dbf52012bfb51705aed0935897d51b31c27fe))
* Batch and Files API ([33920ae](https://github.com/FloopyAI/floopy-rust/commit/33920aec3bc76c29e2c323eaceb97c20d0c4968b))

## [0.2.1](https://github.com/FloopyAI/floopy-rust/compare/floopy-sdk-v0.2.0...floopy-sdk-v0.2.1) (2026-05-17)


### Added

* publish rust sdk ([ff4b895](https://github.com/FloopyAI/floopy-rust/commit/ff4b8952a008b8968da197d6e54243f4c0565247))


### Chore

* release 0.2.0 ([96f2a77](https://github.com/FloopyAI/floopy-rust/commit/96f2a775a14b3240d8fe7f209b565a6fcc29fad9))
* release 0.2.1 ([a4a690d](https://github.com/FloopyAI/floopy-rust/commit/a4a690dfeadf7e35661e789df608e327f05e0ba6))

## [0.2.0](https://github.com/FloopyAI/floopy-rust/compare/floopy-sdk-v0.2.0...floopy-sdk-v0.2.0) (2026-05-17)


### Chore

* release 0.2.0 ([96f2a77](https://github.com/FloopyAI/floopy-rust/commit/96f2a775a14b3240d8fe7f209b565a6fcc29fad9))

## [0.2.0](https://github.com/FloopyAI/floopy-rust/compare/floopy-sdk-v0.1.0...floopy-sdk-v0.2.0) (2026-05-17)


### Added

* publish rust sdk ([ff4b895](https://github.com/FloopyAI/floopy-rust/commit/ff4b8952a008b8968da197d6e54243f4c0565247))

## [Unreleased]

### Added

- Initial Rust SDK: a cheaply-cloneable async `Floopy` client wrapping the
  official `async-openai` crate via a lazy `openai()` delegate, typed
  `FloopyOptions` mapped to `Floopy-*` headers, an internal `reqwest`
  transport with retries/backoff/timeouts honouring `Retry-After`, and the
  `Error` hierarchy. Chat, embeddings, and models reach the gateway 1:1
  with the upstream `async-openai` crate, which is re-exported as
  `floopy::async_openai`.
- Floopy-only resources: `feedback`, `decisions` (+ `Stream`-based `pages`
  / `iter`), `experiments` (with auto `X-Floopy-Confirm`), `constraints`,
  `export` (JSONL streaming with optional trailer capture), `evaluations`,
  `routing.explain`, and `sessions.get`. Each resource is fully typed
  end-to-end and returns the appropriate `Error` variant.
