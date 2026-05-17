//! The [`Floopy`] client and its [`FloopyBuilder`].

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use async_openai::config::OpenAIConfig;
use async_openai::Client as OpenAIClient;

use crate::constants::{DEFAULT_BASE_URL, DEFAULT_MAX_RETRIES, DEFAULT_TIMEOUT};
use crate::error::Result;
use crate::http::{HttpConfig, HttpTransport};
use crate::openai_delegate::new_openai_delegate;
use crate::options::FloopyOptions;
use crate::resources::{
    Constraints, Decisions, Evaluations, Experiments, Export, Feedback, Routing, Sessions,
};

/// The Floopy gateway client.
///
/// Wraps the official [`async_openai`] client (reachable via
/// [`Floopy::openai`]) and exposes typed Floopy-only resources. A `Floopy`
/// is cheap to clone (`Arc` internally) and safe to share across tasks.
#[derive(Clone)]
pub struct Floopy {
    transport: Arc<HttpTransport>,
    openai: Arc<OnceLock<OpenAIClient<OpenAIConfig>>>,
}

impl Floopy {
    /// Construct a client with default settings. `api_key` is required
    /// (starts with `fl_`).
    ///
    /// # Errors
    /// Returns [`crate::Error::Config`] if `api_key` is empty, or
    /// [`crate::Error::Connection`] if the HTTP client cannot be built.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::builder(api_key).build()
    }

    /// Start building a client. Chain setters then call
    /// [`FloopyBuilder::build`].
    #[must_use]
    pub fn builder(api_key: impl Into<String>) -> FloopyBuilder {
        FloopyBuilder::new(api_key)
    }

    /// A lazily-built [`async_openai`] client pre-configured to talk to the
    /// Floopy gateway. `client.openai().chat()` / `.embeddings()` /
    /// `.models()` are 1:1 drop-in replacements for upstream `async-openai`.
    ///
    /// # Panics
    /// Panics only if the delegate cannot be constructed (invalid forwarded
    /// header); this is unreachable for headers the SDK itself produces.
    #[must_use]
    pub fn openai(&self) -> &OpenAIClient<OpenAIConfig> {
        self.openai.get_or_init(|| {
            new_openai_delegate(&self.transport)
                .expect("delegate construction with SDK-produced headers is infallible")
        })
    }

    /// The resolved gateway base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.transport.base_url()
    }

    /// Submit NPS-style feedback for a request/session.
    #[must_use]
    pub fn feedback(&self) -> Feedback {
        Feedback::new(self.transport.clone())
    }

    /// Read the per-request decision audit trail.
    #[must_use]
    pub fn decisions(&self) -> Decisions {
        Decisions::new(self.transport.clone())
    }

    /// Manage A/B routing experiments.
    #[must_use]
    pub fn experiments(&self) -> Experiments {
        Experiments::new(self.transport.clone())
    }

    /// Read and full-replace org spend/rate constraints.
    #[must_use]
    pub fn constraints(&self) -> Constraints {
        Constraints::new(self.transport.clone())
    }

    /// Stream the decision log as typed JSONL.
    #[must_use]
    pub fn export(&self) -> Export {
        Export::new(self.transport.clone())
    }

    /// Run and inspect dataset evaluations.
    #[must_use]
    pub fn evaluations(&self) -> Evaluations {
        Evaluations::new(self.transport.clone())
    }

    /// The routing dry-run (Pro plan).
    #[must_use]
    pub fn routing(&self) -> Routing {
        Routing::new(self.transport.clone())
    }

    /// Restore stored conversations.
    #[must_use]
    pub fn sessions(&self) -> Sessions {
        Sessions::new(self.transport.clone())
    }
}

impl std::fmt::Debug for Floopy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never expose the API key.
        f.debug_struct("Floopy")
            .field("base_url", &self.transport.base_url())
            .finish_non_exhaustive()
    }
}

/// Builder for [`Floopy`]. Created via [`Floopy::builder`].
pub struct FloopyBuilder {
    api_key: String,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    default_headers: HashMap<String, String>,
    options: Option<FloopyOptions>,
    http_client: Option<reqwest::Client>,
}

impl FloopyBuilder {
    fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
            default_headers: HashMap::new(),
            options: None,
            http_client: None,
        }
    }

    /// Override the gateway base URL (default [`DEFAULT_BASE_URL`]). Use for
    /// self-hosted gateways.
    ///
    /// [`DEFAULT_BASE_URL`]: crate::DEFAULT_BASE_URL
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set the default per-request timeout (default 60s).
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the retry budget for transient failures (default 2).
    #[must_use]
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Add a header sent on every Floopy-only request (highest precedence).
    #[must_use]
    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    /// Set the default Floopy options forwarded on every request.
    #[must_use]
    pub fn options(mut self, options: FloopyOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Use a caller-provided [`reqwest::Client`] for Floopy-only requests.
    #[must_use]
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Build the client.
    ///
    /// # Errors
    /// Returns [`crate::Error::Config`] if the API key is empty, or
    /// [`crate::Error::Connection`] if the default HTTP client cannot be
    /// built.
    pub fn build(self) -> Result<Floopy> {
        let transport = HttpTransport::new(HttpConfig {
            api_key: self.api_key,
            base_url: self.base_url,
            timeout: self.timeout,
            max_retries: self.max_retries,
            default_headers: self.default_headers,
            default_options: self.options,
            http_client: self.http_client,
        })?;
        Ok(Floopy {
            transport: Arc::new(transport),
            openai: Arc::new(OnceLock::new()),
        })
    }
}
