# Changelog

All notable changes to `floopy-sdk` (Rust) are documented in this file. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
the project adheres to [Semantic Versioning](https://semver.org/).

Releases are produced by `release-please` from Conventional Commits.

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
