use serde::{Deserialize, Serialize};

/// Arguments for [`crate::resources::Feedback::submit`].
#[derive(Debug, Clone, Serialize)]
pub struct FeedbackSubmitParams {
    /// User rating (e.g. 0-10 / NPS-style).
    pub score: i32,
    /// Whether the response was useful.
    pub useful: bool,
    /// Optional; defaults to the most recent session for the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Response from [`crate::resources::Feedback::submit`].
#[derive(Debug, Clone, Deserialize)]
pub struct FeedbackSubmitResponse {
    /// `true` when feedback for this session was already recorded.
    pub duplicate: bool,
    /// The session the feedback was attached to.
    #[serde(default)]
    pub session_id: Option<String>,
}
