use std::sync::Arc;

use reqwest::Method;

use crate::constants::{
    evaluation_by_id, evaluation_cancel, evaluation_results, ENDPOINT_EVALUATIONS,
};
use crate::error::Result;
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::{EvaluationCreateParams, EvaluationResultsPage, EvaluationRun};

use super::require;

/// Runs and inspects dataset evaluations.
pub struct Evaluations {
    t: Arc<HttpTransport>,
}

impl Evaluations {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Start an evaluation run.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn create(
        &self,
        params: EvaluationCreateParams,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<EvaluationRun> {
        let body =
            serde_json::to_value(&params).map_err(|e| crate::Error::Decode(e.to_string()))?;
        let (data, _) = self
            .t
            .request(
                Method::POST,
                ENDPOINT_EVALUATIONS,
                Some(&body),
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Fetch an evaluation run by id.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn get(
        &self,
        evaluation_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<EvaluationRun> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                &evaluation_by_id(evaluation_id),
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Cancel a running evaluation.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn cancel(
        &self,
        evaluation_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<EvaluationRun> {
        let (data, _) = self
            .t
            .request(
                Method::POST,
                &evaluation_cancel(evaluation_id),
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Fetch a page of scored rows for an evaluation.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn results(
        &self,
        evaluation_id: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<EvaluationResultsPage> {
        let mut query = Vec::new();
        if let Some(limit) = limit {
            query.push(("limit".to_owned(), limit.to_string()));
        }
        if let Some(cursor) = cursor {
            query.push(("cursor".to_owned(), cursor.to_owned()));
        }
        let (data, _) = self
            .t
            .request(
                Method::GET,
                &evaluation_results(evaluation_id),
                None,
                &query,
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }
}
