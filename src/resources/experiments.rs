use std::sync::Arc;

use futures::Stream;
use reqwest::Method;

use crate::constants::{
    experiment_results, experiment_rollback, CONFIRM_EXPERIMENTS, ENDPOINT_EXPERIMENTS,
    HEADER_CONFIRM,
};
use crate::error::Result;
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::{
    Experiment, ExperimentCreateParams, ExperimentListPage, ExperimentListParams, ExperimentResults,
};

use super::require;

/// Manages A/B routing experiments.
pub struct Experiments {
    t: Arc<HttpTransport>,
}

/// Inject `X-Floopy-Confirm: experiments` (gateway SEC-009 requires it on
/// create/rollback) without dropping the caller's overrides.
fn with_confirm(req: Option<RequestOptions>) -> RequestOptions {
    let mut req = req.unwrap_or_default();
    req = req.header(HEADER_CONFIRM, CONFIRM_EXPERIMENTS);
    req
}

impl Experiments {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Fetch a single page of experiments.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn list(
        &self,
        params: &ExperimentListParams,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<ExperimentListPage> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                ENDPOINT_EXPERIMENTS,
                None,
                &params.query(),
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Stream one [`ExperimentListPage`] per round-trip until exhausted.
    pub fn pages(
        &self,
        params: ExperimentListParams,
        req: Option<RequestOptions>,
    ) -> impl Stream<Item = Result<ExperimentListPage>> + Send + 'static {
        let t = self.t.clone();
        async_stream::try_stream! {
            let mut params = params;
            loop {
                let (data, _) = t
                    .request::<ExperimentListPage>(
                        Method::GET,
                        ENDPOINT_EXPERIMENTS,
                        None,
                        &params.query(),
                        req.as_ref(),
                    )
                    .await?;
                let page = require(data)?;
                let next = page.next_cursor.clone();
                let has_more = page.has_more;
                yield page;
                match next {
                    Some(cursor) if has_more && !cursor.is_empty() => {
                        params.cursor = Some(cursor);
                    }
                    _ => break,
                }
            }
        }
    }

    /// Create an experiment. The `X-Floopy-Confirm: experiments` header is
    /// injected automatically.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn create(
        &self,
        params: ExperimentCreateParams,
        req: Option<RequestOptions>,
    ) -> Result<Experiment> {
        let body =
            serde_json::to_value(&params).map_err(|e| crate::Error::Decode(e.to_string()))?;
        let (data, _) = self
            .t
            .request(
                Method::POST,
                ENDPOINT_EXPERIMENTS,
                Some(&body),
                &[],
                Some(&with_confirm(req)),
            )
            .await?;
        require(data)
    }

    /// Roll back an experiment. The `X-Floopy-Confirm: experiments` header
    /// is injected automatically.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn rollback(
        &self,
        experiment_id: &str,
        req: Option<RequestOptions>,
    ) -> Result<Experiment> {
        let (data, _) = self
            .t
            .request(
                Method::POST,
                &experiment_rollback(experiment_id),
                None,
                &[],
                Some(&with_confirm(req)),
            )
            .await?;
        require(data)
    }

    /// Fetch the computed outcome of an experiment.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn results(
        &self,
        experiment_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<ExperimentResults> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                &experiment_results(experiment_id),
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }
}
