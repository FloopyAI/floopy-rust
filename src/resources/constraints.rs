use std::sync::Arc;

use reqwest::Method;

use crate::constants::ENDPOINT_CONSTRAINTS;
use crate::error::Result;
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::OrgConstraints;

use super::require;

/// Reads and full-replaces org constraints.
pub struct Constraints {
    t: Arc<HttpTransport>,
}

impl Constraints {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Return the current org constraints.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn get(&self, req: impl Into<Option<RequestOptions>>) -> Result<OrgConstraints> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                ENDPOINT_CONSTRAINTS,
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Full-replace the org constraints. Any field left `None` is reset to
    /// `null` server-side (matches the gateway's PUT semantics).
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn put(
        &self,
        constraints: &OrgConstraints,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<OrgConstraints> {
        let body =
            serde_json::to_value(constraints).map_err(|e| crate::Error::Decode(e.to_string()))?;
        let (data, _) = self
            .t
            .request(
                Method::PUT,
                ENDPOINT_CONSTRAINTS,
                Some(&body),
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }
}
