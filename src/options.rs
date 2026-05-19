//! Gateway behaviour toggles ([`FloopyOptions`]) and per-call overrides
//! ([`RequestOptions`]). Field names map 1:1 to the other Floopy SDKs.

use std::collections::HashMap;
use std::time::Duration;

use crate::constants::HEADER_FLOOPY_PROVIDER;

/// Cache controls. Maps to the `Floopy-Cache-*` headers. A `None` field is
/// omitted (the gateway default applies).
#[derive(Debug, Clone, Default)]
pub struct CacheOptions {
    /// Toggle the exact + semantic cache for the request.
    pub enabled: Option<bool>,
    /// Maximum number of entries per semantic cache bucket.
    pub bucket_max_size: Option<u32>,
}

/// Gateway behaviour toggles, mapped to `Floopy-*` headers and forwarded to
/// **every** request (both OpenAI-compatible and Floopy-only). Empty / `None`
/// fields are omitted.
#[derive(Debug, Clone, Default)]
pub struct FloopyOptions {
    /// Cache controls. Maps to `Floopy-Cache-*` headers.
    pub cache: Option<CacheOptions>,
    /// Stored prompt id; the gateway resolves it to the active prompt
    /// content. Maps to `Floopy-Prompt-Id`.
    pub prompt_id: Option<String>,
    /// Pinned prompt version. Use with `prompt_id`. Maps to
    /// `Floopy-Prompt-Version`.
    pub prompt_version: Option<String>,
    /// Toggle the LLM firewall pre-check. Maps to
    /// `floopy-llm-security-enabled`.
    pub llm_security_enabled: Option<bool>,
}

/// Per-call overrides, merged on top of the client defaults. Construct with
/// [`RequestOptions::new`] and the builder-style setters.
#[derive(Debug, Clone, Default)]
pub struct RequestOptions {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) options: Option<FloopyOptions>,
}

impl RequestOptions {
    /// An empty set of per-call overrides.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add (or replace) a single per-call header. Highest precedence.
    #[must_use]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Select the upstream the gateway forwards a Batch/Files request to
    /// (the `floopy-provider` header). A batch carries no model up front so
    /// the provider cannot be inferred — set this, or rely on the key's
    /// single configured provider. No-op for other resources.
    #[must_use]
    pub fn provider(self, provider: impl Into<String>) -> Self {
        self.header(HEADER_FLOOPY_PROVIDER, provider)
    }

    /// Override the client timeout for this call.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Override the Floopy options for this call.
    #[must_use]
    pub fn options(mut self, options: FloopyOptions) -> Self {
        self.options = Some(options);
        self
    }
}
