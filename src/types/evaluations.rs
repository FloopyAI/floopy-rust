use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A dataset evaluation run.
#[derive(Debug, Clone, Deserialize)]
pub struct EvaluationRun {
    /// Run id.
    pub id: String,
    /// Dataset under evaluation.
    pub dataset_id: String,
    /// Model under evaluation.
    pub model: String,
    /// Stored prompt id, if used.
    pub prompt_id: Option<String>,
    /// Lifecycle state (`pending`/`running`/`completed`/`failed`/`cancelled`).
    pub status: String,
    /// Free-form run configuration.
    pub config: Option<Value>,
    /// RFC3339 creation timestamp.
    pub created_at: String,
    /// RFC3339 start timestamp, if started.
    pub started_at: Option<String>,
    /// RFC3339 finish timestamp, if finished.
    pub finished_at: Option<String>,
}

/// One scored row of an evaluation.
#[derive(Debug, Clone, Deserialize)]
pub struct EvaluationResultRow {
    /// Row id.
    pub id: String,
    /// Owning run id.
    pub run_id: String,
    /// Dataset input id.
    pub input_id: String,
    /// Model output.
    pub output: String,
    /// Score, if computed.
    pub score: Option<f64>,
    /// Free-form per-row metadata.
    pub metadata: Option<Value>,
    /// RFC3339 creation timestamp.
    pub created_at: String,
}

/// One page of [`crate::resources::Evaluations::results`].
#[derive(Debug, Clone, Deserialize)]
pub struct EvaluationResultsPage {
    /// Result rows in this page.
    pub items: Vec<EvaluationResultRow>,
    /// Cursor for the next page, if any.
    pub next_cursor: Option<String>,
    /// Whether more pages follow.
    pub has_more: bool,
}

/// Arguments for [`crate::resources::Evaluations::create`].
#[derive(Debug, Clone, Serialize)]
pub struct EvaluationCreateParams {
    /// Dataset to evaluate.
    pub dataset_id: String,
    /// Model to evaluate.
    pub model: String,
    /// Optional stored prompt id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_id: Option<String>,
    /// Optional free-form run configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
}
