use serde::{Deserialize, Serialize};

/// An A/B routing experiment.
#[derive(Debug, Clone, Deserialize)]
pub struct Experiment {
    /// Experiment id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Lifecycle state (`active`, `rolled_back`, `completed`).
    pub status: String,
    /// Routing rule id for variant A.
    pub variant_a_routing_rule_id: String,
    /// Routing rule id for variant B.
    pub variant_b_routing_rule_id: String,
    /// Percentage of traffic on variant B.
    pub split_percentage: i32,
    /// RFC3339 creation timestamp.
    pub created_at: String,
    /// RFC3339 rollback timestamp, if rolled back.
    pub rolled_back_at: Option<String>,
}

/// One page of [`crate::resources::Experiments::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentListPage {
    /// Experiments in this page.
    pub items: Vec<Experiment>,
    /// Cursor for the next page, if any.
    pub next_cursor: Option<String>,
    /// Whether more pages follow.
    pub has_more: bool,
}

/// Aggregated metrics for one experiment variant.
#[derive(Debug, Clone, Deserialize)]
pub struct VariantResults {
    /// Routing rule id for this variant.
    pub routing_rule_id: String,
    /// Number of samples observed.
    pub sample_size: i64,
    /// Fraction of successful requests.
    pub success_rate: f64,
    /// Mean latency in milliseconds.
    pub average_latency_ms: f64,
    /// Mean cost in micro-USD.
    pub average_cost_micro_usd: f64,
}

/// Computed outcome of an experiment.
#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentResults {
    /// Experiment id.
    pub experiment_id: String,
    /// Variant A metrics.
    pub variant_a: VariantResults,
    /// Variant B metrics.
    pub variant_b: VariantResults,
    /// `"A"`, `"B"`, `"tie"`, or `None` when undecided.
    pub winner: Option<String>,
    /// RFC3339 timestamp the results were computed.
    pub computed_at: String,
}

/// Filters for [`crate::resources::Experiments::list`].
#[derive(Debug, Clone, Default)]
pub struct ExperimentListParams {
    /// Restrict to one lifecycle state.
    pub status: Option<String>,
    /// Inclusive lower bound (RFC3339).
    pub from: Option<String>,
    /// Inclusive upper bound (RFC3339).
    pub to: Option<String>,
    /// Page size.
    pub limit: Option<u32>,
    /// Opaque pagination cursor.
    pub cursor: Option<String>,
}

impl ExperimentListParams {
    pub(crate) fn query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(v) = &self.status {
            q.push(("status".to_owned(), v.clone()));
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

/// Arguments for [`crate::resources::Experiments::create`].
#[derive(Debug, Clone, Serialize)]
pub struct ExperimentCreateParams {
    /// Display name.
    pub name: String,
    /// Routing rule id for variant A.
    pub variant_a_routing_rule_id: String,
    /// Routing rule id for variant B.
    pub variant_b_routing_rule_id: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional percentage of traffic on variant B.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split_percentage: Option<i32>,
}
