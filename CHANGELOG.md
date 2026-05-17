# Changelog

All notable changes to `floopy-sdk` (Rust) are documented in this file. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
the project adheres to [Semantic Versioning](https://semver.org/).

Releases are produced by `release-please` from Conventional Commits.

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
