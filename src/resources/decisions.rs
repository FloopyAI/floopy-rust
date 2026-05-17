use std::sync::Arc;

use futures::Stream;
use reqwest::Method;

use crate::constants::{decision_by_id, ENDPOINT_DECISIONS};
use crate::error::Result;
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::{Decision, DecisionListPage, DecisionListParams};

use super::require;

/// Reads the decision audit trail.
pub struct Decisions {
    t: Arc<HttpTransport>,
}

impl Decisions {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Fetch one decision by request id.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn get(
        &self,
        request_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<Decision> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                &decision_by_id(request_id),
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Fetch a single page of decisions.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn list(
        &self,
        params: &DecisionListParams,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<DecisionListPage> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                ENDPOINT_DECISIONS,
                None,
                &params.query(),
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Stream one [`DecisionListPage`] per network round-trip until the
    /// gateway reports no more pages.
    pub fn pages(
        &self,
        params: DecisionListParams,
        req: Option<RequestOptions>,
    ) -> impl Stream<Item = Result<DecisionListPage>> + Send + 'static {
        let t = self.t.clone();
        async_stream::try_stream! {
            let mut params = params;
            loop {
                let (data, _) = t
                    .request::<DecisionListPage>(
                        Method::GET,
                        ENDPOINT_DECISIONS,
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

    /// Stream every decision across all pages.
    pub fn iter(
        &self,
        params: DecisionListParams,
        req: Option<RequestOptions>,
    ) -> impl Stream<Item = Result<Decision>> + Send + 'static {
        let pages = self.pages(params, req);
        async_stream::try_stream! {
            futures::pin_mut!(pages);
            while let Some(page) = futures::StreamExt::next(&mut pages).await {
                let page = page?;
                for decision in page.items {
                    yield decision;
                }
            }
        }
    }
}
