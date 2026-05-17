//! Build an [`async_openai`] client pre-pointed at the Floopy gateway.
//!
//! `async-openai` manages auth, content-type and user-agent itself, so the
//! forwarded set is just the `Floopy-*` toggles plus the SDK marker header;
//! they ride along on every OpenAI-compatible call.

use async_openai::config::OpenAIConfig;
use async_openai::Client as OpenAIClient;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::constants::{HEADER_FLOOPY_SDK, USER_AGENT_PREFIX};
use crate::error::{Error, Result};
use crate::http::HttpTransport;

pub(crate) fn new_openai_delegate(transport: &HttpTransport) -> Result<OpenAIClient<OpenAIConfig>> {
    let mut headers = transport.delegate_headers();
    headers.insert(
        HEADER_FLOOPY_SDK.to_owned(),
        format!("{}/{}", USER_AGENT_PREFIX, env!("CARGO_PKG_VERSION")),
    );

    let mut header_map = HeaderMap::with_capacity(headers.len());
    for (k, v) in &headers {
        let name = HeaderName::from_bytes(k.as_bytes())
            .map_err(|e| Error::Config(format!("invalid delegate header {k:?}: {e}")))?;
        let value = HeaderValue::from_str(v)
            .map_err(|e| Error::Config(format!("invalid delegate header value for {k:?}: {e}")))?;
        header_map.insert(name, value);
    }

    let http = reqwest::Client::builder()
        .default_headers(header_map)
        .build()
        .map_err(Error::Connection)?;

    let config = OpenAIConfig::new()
        .with_api_base(transport.base_url())
        .with_api_key(transport.api_key());

    Ok(OpenAIClient::with_config(config).with_http_client(http))
}
