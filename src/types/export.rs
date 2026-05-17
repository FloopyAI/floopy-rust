use serde::Deserialize;

/// One row of the decision export.
#[derive(Debug, Clone, Deserialize)]
pub struct ExportedDecisionRow {
    /// The originating request id.
    pub request_id: String,
    /// Session the request belonged to, if any.
    pub session_id: Option<String>,
    /// Owning organization id.
    pub organization_id: String,
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
    /// Whether caching was enabled.
    pub cache_enabled: Option<bool>,
    /// Detected threat label, if any.
    pub threat: Option<String>,
    /// RFC3339 creation timestamp.
    pub created_at: String,
}

/// The terminal record of the JSONL export: how many rows were emitted and
/// whether the result was truncated.
#[derive(Debug, Clone, Deserialize)]
pub struct ExportTrailer {
    /// Number of rows emitted before the trailer.
    pub rows_emitted: i64,
    /// Whether the export was truncated server-side.
    pub truncated: bool,
    /// Reason for truncation, if any.
    pub reason: Option<String>,
}

/// Selects the export window and (server-side) format. The SDK always parses
/// the JSONL stream; `format` is accepted for parity with the gateway.
#[derive(Debug, Clone)]
pub struct ExportDecisionsParams {
    /// Inclusive lower bound (RFC3339). Required.
    pub from: String,
    /// Inclusive upper bound (RFC3339). Required.
    pub to: String,
    /// Optional server-side format hint (`"jsonl"` / `"csv"`).
    pub format: Option<String>,
}

impl ExportDecisionsParams {
    /// A window with the default (JSONL) format.
    #[must_use]
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            format: None,
        }
    }

    pub(crate) fn query(&self) -> Vec<(String, String)> {
        let mut q = vec![
            ("from".to_owned(), self.from.clone()),
            ("to".to_owned(), self.to.clone()),
        ];
        if let Some(f) = &self.format {
            q.push(("format".to_owned(), f.clone()));
        }
        q
    }
}
