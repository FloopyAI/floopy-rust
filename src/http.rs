//! Internal HTTP transport for Floopy-only endpoints.
//!
//! Bearer auth, `Floopy-*` header injection, bounded retries with
//! exponential backoff + jitter (`Retry-After` honoured), per-call timeouts,
//! request-id capture, and mapping of non-2xx responses to [`Error`].
//!
//! The OpenAI-compatible surface (chat/embeddings/models) does *not* go
//! through here — it is delegated to [`async_openai`].
//!
//! Security: the API key only ever appears in the `Authorization` header;
//! the SDK never logs request or response bodies.

use std::collections::HashMap;
use std::time::Duration;

use futures::Stream;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::constants::{HEADER_REQUEST_ID, USER_AGENT_PREFIX};
use crate::error::{from_status, Error, Result};
use crate::headers::{build_floopy_headers, merge_headers};
use crate::options::{FloopyOptions, RequestOptions};

const RETRYABLE_STATUS: &[u16] = &[408, 409, 425, 429, 500, 502, 503, 504];

pub(crate) struct HttpConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub default_headers: HashMap<String, String>,
    pub default_options: Option<FloopyOptions>,
    pub http_client: Option<reqwest::Client>,
}

pub(crate) struct HttpTransport {
    api_key: String,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    default_headers: HashMap<String, String>,
    default_options: Option<FloopyOptions>,
    client: reqwest::Client,
}

impl HttpTransport {
    pub(crate) fn new(cfg: HttpConfig) -> Result<Self> {
        if cfg.api_key.is_empty() {
            return Err(Error::Config(
                "api key is required to construct a Floopy client".to_owned(),
            ));
        }
        let client = match cfg.http_client {
            Some(c) => c,
            None => reqwest::Client::builder()
                .build()
                .map_err(Error::Connection)?,
        };
        Ok(Self {
            api_key: cfg.api_key,
            base_url: cfg.base_url.trim_end_matches('/').to_owned(),
            timeout: cfg.timeout,
            max_retries: cfg.max_retries,
            default_headers: cfg.default_headers,
            default_options: cfg.default_options,
            client,
        })
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    fn user_agent(&self) -> String {
        format!(
            "{}/{} rust/{}",
            USER_AGENT_PREFIX,
            env!("CARGO_PKG_VERSION"),
            option_env!("CARGO_PKG_RUST_VERSION").unwrap_or("unknown")
        )
    }

    /// Header set forwarded to the OpenAI delegate (Floopy toggles +
    /// caller defaults). Auth and user-agent are handled by `async-openai`.
    pub(crate) fn delegate_headers(&self) -> HashMap<String, String> {
        merge_headers([
            &self.default_headers,
            &build_floopy_headers(self.default_options.as_ref()),
        ])
    }

    fn request_headers(&self, req: Option<&RequestOptions>) -> HashMap<String, String> {
        let per_call_opts = req.and_then(|r| r.options.as_ref());
        let per_call_headers = req.map(|r| r.headers.clone()).unwrap_or_default();
        merge_headers([
            &self.default_headers,
            &build_floopy_headers(self.default_options.as_ref()),
            &build_floopy_headers(per_call_opts),
            &per_call_headers,
        ])
    }

    fn build_url(&self, path: &str, query: &[(String, String)]) -> String {
        let normalized = if path.starts_with('/') {
            path.to_owned()
        } else {
            format!("/{path}")
        };
        let mut url = format!("{}{normalized}", self.base_url);
        if !query.is_empty() {
            let qs = serde_urlencoded_lite(query);
            if !qs.is_empty() {
                url.push('?');
                url.push_str(&qs);
            }
        }
        url
    }

    fn timeout_for(&self, req: Option<&RequestOptions>) -> Duration {
        req.and_then(|r| r.timeout).unwrap_or(self.timeout)
    }

    fn header_map(&self, headers: &HashMap<String, String>) -> HeaderMap {
        let mut map = HeaderMap::with_capacity(headers.len());
        for (k, v) in headers {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                map.insert(name, value);
            }
        }
        map
    }

    /// Issue a request with retries and decode a 2xx JSON body into `T`.
    /// Returns the decoded value (or `None` on 204 / empty body) and the
    /// `X-Request-Id` header when present.
    pub(crate) async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
        query: &[(String, String)],
        req: Option<&RequestOptions>,
    ) -> Result<(Option<T>, Option<String>)> {
        let url = self.build_url(path, query);
        let header_map = self.header_map(&self.request_headers(req));
        let timeout = self.timeout_for(req);
        let body_bytes = match body {
            Some(v) => Some(serde_json::to_vec(v).map_err(|e| Error::Decode(e.to_string()))?),
            None => None,
        };

        let mut attempt: u32 = 0;
        loop {
            let mut builder = self
                .client
                .request(method.clone(), &url)
                .headers(header_map.clone())
                .bearer_auth(&self.api_key)
                .header(reqwest::header::USER_AGENT, self.user_agent())
                .timeout(timeout);
            if let Some(bytes) = &body_bytes {
                builder = builder
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(bytes.clone());
            }

            let response = match builder.send().await {
                Ok(r) => r,
                Err(err) => {
                    if err.is_timeout() {
                        return Err(Error::Timeout(format!(
                            "request timed out after {timeout:?}"
                        )));
                    }
                    if attempt < self.max_retries {
                        backoff_sleep(attempt, None).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(Error::Connection(err));
                }
            };

            let status = response.status();
            let request_id = response
                .headers()
                .get(HEADER_REQUEST_ID)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned);

            if status.is_success() {
                if status.as_u16() == 204 {
                    return Ok((None, request_id));
                }
                let text = response.text().await.map_err(Error::Connection)?;
                if text.is_empty() {
                    return Ok((None, request_id));
                }
                let value =
                    serde_json::from_str::<T>(&text).map_err(|e| Error::Decode(e.to_string()))?;
                return Ok((Some(value), request_id));
            }

            let code = status.as_u16();
            let retry_after = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            if attempt < self.max_retries && RETRYABLE_STATUS.contains(&code) {
                backoff_sleep(attempt, retry_after).await;
                attempt += 1;
                continue;
            }

            let raw = response.text().await.unwrap_or_default();
            let parsed: Option<Value> = if raw.is_empty() {
                None
            } else {
                serde_json::from_str(&raw).ok().or(Some(Value::String(raw)))
            };
            return Err(from_status(code, parsed, request_id, retry_after));
        }
    }

    /// Stream a JSONL response, yielding one parsed JSON value per non-empty
    /// line. Errors surface as `Err` items.
    pub(crate) async fn stream_lines(
        &self,
        method: Method,
        path: &str,
        query: &[(String, String)],
        req: Option<&RequestOptions>,
    ) -> Result<impl Stream<Item = Result<Value>> + Send> {
        use futures::StreamExt;

        let url = self.build_url(path, query);
        let header_map = self.header_map(&self.request_headers(req));
        let timeout = self.timeout_for(req);

        let response = self
            .client
            .request(method, &url)
            .headers(header_map)
            .bearer_auth(&self.api_key)
            .header(reqwest::header::USER_AGENT, self.user_agent())
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    Error::Timeout("request timed out while streaming".to_owned())
                } else {
                    Error::Connection(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let request_id = response
                .headers()
                .get(HEADER_REQUEST_ID)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned);
            let raw = response.text().await.unwrap_or_default();
            let parsed = if raw.is_empty() {
                None
            } else {
                serde_json::from_str(&raw).ok().or(Some(Value::String(raw)))
            };
            return Err(from_status(status.as_u16(), parsed, request_id, None));
        }

        let mut bytes = response.bytes_stream();
        let stream = async_stream::try_stream! {
            let mut buf: Vec<u8> = Vec::new();
            while let Some(chunk) = bytes.next().await {
                let chunk = chunk.map_err(Error::Connection)?;
                buf.extend_from_slice(&chunk);
                while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    let line: Vec<u8> = buf.drain(..=pos).collect();
                    let trimmed = trim_ascii(&line);
                    if trimmed.is_empty() {
                        continue;
                    }
                    let value: Value = serde_json::from_slice(trimmed)
                        .map_err(|e| Error::Decode(e.to_string()))?;
                    yield value;
                }
            }
            let trimmed = trim_ascii(&buf);
            if !trimmed.is_empty() {
                let value: Value = serde_json::from_slice(trimmed)
                    .map_err(|e| Error::Decode(e.to_string()))?;
                yield value;
            }
        };
        Ok(stream)
    }
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes.iter().position(|b| !b.is_ascii_whitespace());
    let Some(start) = start else { return &[] };
    let end = bytes
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())
        .unwrap_or(start);
    &bytes[start..=end]
}

async fn backoff_sleep(attempt: u32, retry_after_seconds: Option<u64>) {
    if let Some(secs) = retry_after_seconds {
        if secs > 0 {
            tokio::time::sleep(Duration::from_secs(secs)).await;
            return;
        }
    }
    let base = Duration::from_millis(250) * 2u32.pow(attempt);
    // Deterministic jitter from the system clock's sub-nanos — no extra
    // dependency just to perturb a backoff.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let jitter = base.mul_f64(0.25 * (f64::from(nanos) / 1_000_000_000.0));
    tokio::time::sleep(base + jitter).await;
}

/// Minimal `application/x-www-form-urlencoded` query encoder. Avoids pulling
/// `serde_urlencoded` just for a handful of string pairs.
fn serde_urlencoded_lite(pairs: &[(String, String)]) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    pairs
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| {
            format!(
                "{}={}",
                utf8_percent_encode(k, NON_ALPHANUMERIC),
                utf8_percent_encode(v, NON_ALPHANUMERIC)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}
