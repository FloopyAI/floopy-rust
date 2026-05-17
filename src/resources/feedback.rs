use std::sync::Arc;

use reqwest::Method;

use crate::constants::ENDPOINT_FEEDBACK;
use crate::error::Result;
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::{FeedbackSubmitParams, FeedbackSubmitResponse};

use super::require;

/// Submits feedback for a completed request or session.
pub struct Feedback {
    t: Arc<HttpTransport>,
}

impl Feedback {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Submit feedback. `session_id` is usually the chat completion id.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn submit(
        &self,
        params: FeedbackSubmitParams,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<FeedbackSubmitResponse> {
        let body =
            serde_json::to_value(&params).map_err(|e| crate::Error::Decode(e.to_string()))?;
        let (data, _) = self
            .t
            .request(
                Method::POST,
                ENDPOINT_FEEDBACK,
                Some(&body),
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }
}
