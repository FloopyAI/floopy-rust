use std::sync::Arc;

use reqwest::Method;

use crate::constants::session_by_id;
use crate::error::Result;
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::Session;

use super::require;

/// Restores stored conversations.
pub struct Sessions {
    t: Arc<HttpTransport>,
}

impl Sessions {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Restore a stored conversation by its session id (the value sent on
    /// the `floopy-session-id` header at request time). Scoped to the API
    /// key's organization.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn get(
        &self,
        session_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<Session> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                &session_by_id(session_id),
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }
}
