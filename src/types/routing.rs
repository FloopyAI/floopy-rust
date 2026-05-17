use std::collections::HashMap;

use async_openai::types::chat::ChatCompletionRequestMessage;
use serde::{Deserialize, Serialize};

/// Outcome of a routing dry-run.
#[derive(Debug, Clone, Deserialize)]
pub struct RoutingExplainResult {
    /// Provider/model the gateway would route to, or `None` when the
    /// firewall blocks the request.
    pub would_select: Option<HashMap<String, String>>,
    /// Firewall verdict (`"allow"` / `"block_input"`).
    pub firewall_decision: String,
    /// Firewall reasoning, if any.
    pub reasoning: Option<String>,
    /// Matched routing rule id, if any.
    pub routing_rule_id: Option<String>,
}

/// Arguments for [`crate::resources::Routing::explain`]. `messages` reuses
/// the `async-openai` request-message type, so a request can be dry-run
/// before it is sent.
#[derive(Debug, Clone, Serialize)]
pub struct RoutingExplainParams {
    /// Model to plan for.
    pub model: String,
    /// Conversation to plan for.
    pub messages: Vec<ChatCompletionRequestMessage>,
    /// Optional sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Optional max tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Optional nucleus-sampling top-p.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
}

impl RoutingExplainParams {
    /// A dry-run for `model` over `messages` with default sampling.
    #[must_use]
    pub fn new(model: impl Into<String>, messages: Vec<ChatCompletionRequestMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            top_p: None,
        }
    }
}
