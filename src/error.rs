//! The [`Error`] hierarchy. One variant per failure mode the gateway can
//! signal, plus transport-level timeout/connection errors. Mirrors the
//! `FloopyError` hierarchy in the Node/Python/Go SDKs.
//!
//! Errors raised by the OpenAI-compatible surface (chat/embeddings/models)
//! come from the underlying [`async_openai`] crate
//! ([`async_openai::error::OpenAIError`]), *not* this type.

use std::fmt;

/// Structured detail attached to every gateway-originated [`Error`]. It is
/// always boxed inside the [`Error`] variants to keep the error type small.
#[derive(Debug, Clone)]
pub struct ErrorDetails {
    /// Human-readable message (gateway-provided when available, otherwise
    /// `"HTTP <status>"`).
    pub message: String,
    /// HTTP status code, when the error originated from a response.
    pub status: Option<u16>,
    /// Gateway error code, when present.
    pub code: Option<String>,
    /// Value of the `X-Request-Id` response header, when present.
    pub request_id: Option<String>,
    /// The plan capability the request needed (on a [`Error::Plan`]).
    pub feature: Option<String>,
    /// `Retry-After` value in seconds (on a [`Error::RateLimit`]).
    pub retry_after_seconds: Option<u64>,
    /// Parsed error body, when present.
    pub body: Option<serde_json::Value>,
}

impl fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(status) = self.status {
            write!(f, " (status={status}")?;
            if let Some(rid) = &self.request_id {
                write!(f, " request_id={rid}")?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

/// Every error returned by a Floopy-only resource.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// HTTP 401, or 403 without a `feature` field.
    #[error("authentication failed: {0}")]
    Auth(Box<ErrorDetails>),

    /// HTTP 403 with a `feature` field: the current plan does not include
    /// the requested capability (see [`ErrorDetails::feature`]).
    #[error("plan does not allow this feature: {0}")]
    Plan(Box<ErrorDetails>),

    /// HTTP 429 (see [`ErrorDetails::retry_after_seconds`]).
    #[error("rate limit exceeded: {0}")]
    RateLimit(Box<ErrorDetails>),

    /// HTTP 400.
    #[error("invalid request: {0}")]
    Validation(Box<ErrorDetails>),

    /// HTTP 404.
    #[error("not found: {0}")]
    NotFound(Box<ErrorDetails>),

    /// HTTP 409.
    #[error("conflict: {0}")]
    Conflict(Box<ErrorDetails>),

    /// HTTP 5xx.
    #[error("gateway error: {0}")]
    Server(Box<ErrorDetails>),

    /// Any other non-2xx response.
    #[error("api error: {0}")]
    Api(Box<ErrorDetails>),

    /// The request exceeded its deadline.
    #[error("request timed out: {0}")]
    Timeout(String),

    /// A network failure talking to the gateway.
    #[error("connection error: {0}")]
    Connection(#[source] reqwest::Error),

    /// A 2xx response whose body could not be decoded.
    #[error("failed to decode response: {0}")]
    Decode(String),

    /// Invalid client configuration (e.g. an empty API key).
    #[error("invalid configuration: {0}")]
    Config(String),
}

impl Error {
    /// The structured detail for gateway-originated errors, if any.
    #[must_use]
    pub fn details(&self) -> Option<&ErrorDetails> {
        match self {
            Error::Auth(d)
            | Error::Plan(d)
            | Error::RateLimit(d)
            | Error::Validation(d)
            | Error::NotFound(d)
            | Error::Conflict(d)
            | Error::Server(d)
            | Error::Api(d) => Some(d),
            Error::Timeout(_) | Error::Connection(_) | Error::Decode(_) | Error::Config(_) => None,
        }
    }

    /// The HTTP status code, when the error originated from a response.
    #[must_use]
    pub fn status(&self) -> Option<u16> {
        self.details().and_then(|d| d.status)
    }

    /// The `X-Request-Id` of the failed request, when present.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        self.details().and_then(|d| d.request_id.as_deref())
    }

    /// The plan capability the request needed, for [`Error::Plan`].
    #[must_use]
    pub fn feature(&self) -> Option<&str> {
        self.details().and_then(|d| d.feature.as_deref())
    }

    /// The `Retry-After` hint in seconds, for [`Error::RateLimit`].
    #[must_use]
    pub fn retry_after_seconds(&self) -> Option<u64> {
        self.details().and_then(|d| d.retry_after_seconds)
    }
}

/// Map an HTTP status + parsed error body to the right [`Error`]. The
/// gateway returns `{"error": {"code", "message", "feature"}}`; a plain
/// body is preserved on [`ErrorDetails::body`] and surfaced generically.
pub(crate) fn from_status(
    status: u16,
    body: Option<serde_json::Value>,
    request_id: Option<String>,
    retry_after_seconds: Option<u64>,
) -> Error {
    let err_obj = body.as_ref().and_then(|b| b.get("error"));
    let message = err_obj
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("HTTP {status}"));
    let code = err_obj
        .and_then(|e| e.get("code"))
        .and_then(|c| c.as_str())
        .map(str::to_owned);
    let feature = err_obj
        .and_then(|e| e.get("feature"))
        .and_then(|f| f.as_str())
        .map(str::to_owned);

    let details = Box::new(ErrorDetails {
        message,
        status: Some(status),
        code,
        request_id,
        feature: feature.clone(),
        retry_after_seconds,
        body,
    });

    match status {
        400 => Error::Validation(details),
        401 => Error::Auth(details),
        403 if feature.is_some() => Error::Plan(details),
        403 => Error::Auth(details),
        404 => Error::NotFound(details),
        409 => Error::Conflict(details),
        429 => Error::RateLimit(details),
        s if s >= 500 => Error::Server(details),
        _ => Error::Api(details),
    }
}

/// Convenience alias for results returned by Floopy-only resources.
pub type Result<T> = std::result::Result<T, Error>;
