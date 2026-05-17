use std::sync::Arc;

use reqwest::Method;

use crate::constants::ENDPOINT_ROUTING_EXPLAIN;
use crate::error::Result;
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::{RoutingExplainParams, RoutingExplainResult};

use super::require;

/// The routing dry-run (Pro plan).
pub struct Routing {
    t: Arc<HttpTransport>,
}

impl Routing {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Run the router and firewall without calling a provider.
    /// `would_select` is `None` when the firewall would block the request.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn explain(
        &self,
        params: RoutingExplainParams,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<RoutingExplainResult> {
        let body =
            serde_json::to_value(&params).map_err(|e| crate::Error::Decode(e.to_string()))?;
        let (data, _) = self
            .t
            .request(
                Method::POST,
                ENDPOINT_ROUTING_EXPLAIN,
                Some(&body),
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }
}
