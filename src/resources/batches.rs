use std::sync::Arc;

use reqwest::Method;

use crate::constants::{batch_by_id, batch_cancel, ENDPOINT_BATCHES};
use crate::error::Result;
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::{Batch, BatchCreateParams, BatchList, BatchListParams};

use super::require;

/// Batches API — create/list/retrieve/cancel asynchronous batch jobs.
///
/// A batch carries no model up front, so select the upstream with
/// [`RequestOptions::provider`] (the `floopy-provider` header).
pub struct Batches {
    t: Arc<HttpTransport>,
}

impl Batches {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Create a batch from a previously uploaded input file.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn create(
        &self,
        params: BatchCreateParams,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<Batch> {
        let body =
            serde_json::to_value(&params).map_err(|e| crate::Error::Decode(e.to_string()))?;
        let (data, _) = self
            .t
            .request(
                Method::POST,
                ENDPOINT_BATCHES,
                Some(&body),
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// List batches for the organization.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn list(
        &self,
        params: BatchListParams,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<BatchList> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                ENDPOINT_BATCHES,
                None,
                &params.query(),
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Retrieve a single batch (poll its status).
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn retrieve(
        &self,
        batch_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<Batch> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                &batch_by_id(batch_id),
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Request cancellation of an in-progress batch.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn cancel(
        &self,
        batch_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<Batch> {
        let (data, _) = self
            .t
            .request(
                Method::POST,
                &batch_cancel(batch_id),
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }
}
