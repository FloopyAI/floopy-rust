use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Per-batch progress summary.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchRequestCounts {
    /// Total requests in the batch.
    #[serde(default)]
    pub total: Option<i64>,
    /// Requests that completed successfully.
    #[serde(default)]
    pub completed: Option<i64>,
    /// Requests that failed.
    #[serde(default)]
    pub failed: Option<i64>,
}

/// An asynchronous batch job. Mirrors the OpenAI batch object; the gateway
/// forwards batch traffic verbatim.
#[derive(Debug, Clone, Deserialize)]
pub struct Batch {
    /// Provider-issued batch id.
    pub id: String,
    /// Object type (usually `"batch"`).
    #[serde(default)]
    pub object: Option<String>,
    /// The endpoint the batch targets.
    #[serde(default)]
    pub endpoint: Option<String>,
    /// Lifecycle status.
    #[serde(default)]
    pub status: Option<String>,
    /// Input file id.
    #[serde(default)]
    pub input_file_id: Option<String>,
    /// Output file id (set when completed).
    #[serde(default)]
    pub output_file_id: Option<String>,
    /// Error file id (set when there were failures).
    #[serde(default)]
    pub error_file_id: Option<String>,
    /// Unix creation timestamp.
    #[serde(default)]
    pub created_at: Option<i64>,
    /// Unix completion timestamp.
    #[serde(default)]
    pub completed_at: Option<i64>,
    /// Progress counts.
    #[serde(default)]
    pub request_counts: Option<BatchRequestCounts>,
}

/// Response of [`crate::resources::Batches::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct BatchList {
    /// Object type (usually `"list"`).
    #[serde(default)]
    pub object: Option<String>,
    /// The batches.
    #[serde(default)]
    pub data: Vec<Batch>,
    /// Whether more pages are available.
    #[serde(default)]
    pub has_more: Option<bool>,
}

/// Arguments for [`crate::resources::Batches::create`].
#[derive(Debug, Clone, Serialize)]
pub struct BatchCreateParams {
    /// Id of a previously uploaded input file.
    pub input_file_id: String,
    /// The endpoint each line targets (e.g. `/v1/chat/completions`).
    pub endpoint: String,
    /// Completion window (e.g. `"24h"`).
    pub completion_window: String,
    /// Optional free-form metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Filters for [`crate::resources::Batches::list`].
#[derive(Debug, Clone, Default)]
pub struct BatchListParams {
    /// Page size.
    pub limit: Option<u32>,
    /// Cursor: return batches after this id.
    pub after: Option<String>,
}

impl BatchListParams {
    pub(crate) fn query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(l) = self.limit {
            q.push(("limit".to_owned(), l.to_string()));
        }
        if let Some(a) = &self.after {
            q.push(("after".to_owned(), a.clone()));
        }
        q
    }
}
