use serde::Deserialize;
use serde_json::Value;

/// One row of the per-request decision audit trail. Nullable gateway fields
/// are `Option` (`None` == JSON `null`).
#[derive(Debug, Clone, Deserialize)]
pub struct Decision {
    /// The originating request id.
    pub request_id: String,
    /// Session the request belonged to, if any.
    pub session_id: Option<String>,
    /// RFC3339 timestamp of the originating request.
    pub request_created_at: String,
    /// Resolved upstream provider.
    pub provider: Option<String>,
    /// Resolved upstream model.
    pub model: Option<String>,
    /// Terminal status of the request.
    pub status: String,
    /// End-to-end latency in milliseconds.
    pub latency_ms: Option<i64>,
    /// Cost in micro-USD.
    pub cost_micro_usd: Option<i64>,
    /// Whether caching was enabled for the request.
    pub cache_enabled: Option<bool>,
    /// Detected threat label, if the firewall flagged the request.
    pub threat: Option<String>,
    /// Opaque routing/decision trace.
    pub decision_trace: Option<Value>,
    /// Routing confidence score.
    pub confidence: Option<f64>,
    /// Why that confidence was assigned.
    pub confidence_reason: Option<String>,
    /// Human-readable explanation of the decision.
    pub explanation: Option<String>,
}

/// One page of [`crate::resources::Decisions::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct DecisionListPage {
    /// Decisions in this page.
    pub items: Vec<Decision>,
    /// Cursor for the next page, if any.
    pub next_cursor: Option<String>,
    /// Whether more pages follow.
    pub has_more: bool,
}

/// Filters for [`crate::resources::Decisions::list`] / `pages` / `iter`.
/// Default is "no filter".
#[derive(Debug, Clone, Default)]
pub struct DecisionListParams {
    /// Restrict to one session.
    pub session_id: Option<String>,
    /// Inclusive lower bound (RFC3339).
    pub from: Option<String>,
    /// Inclusive upper bound (RFC3339).
    pub to: Option<String>,
    /// Page size.
    pub limit: Option<u32>,
    /// Opaque pagination cursor.
    pub cursor: Option<String>,
}

impl DecisionListParams {
    pub(crate) fn query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(v) = &self.session_id {
            q.push(("session_id".to_owned(), v.clone()));
        }
        if let Some(v) = &self.from {
            q.push(("from".to_owned(), v.clone()));
        }
        if let Some(v) = &self.to {
            q.push(("to".to_owned(), v.clone()));
        }
        if let Some(v) = self.limit {
            q.push(("limit".to_owned(), v.to_string()));
        }
        if let Some(v) = &self.cursor {
            q.push(("cursor".to_owned(), v.clone()));
        }
        q
    }
}
