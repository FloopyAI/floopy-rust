use serde::{Deserialize, Serialize};

/// Org-wide spend and rate constraints. All fields are `Option`: `None`
/// means "no limit" — and, on
/// [`put`](crate::resources::Constraints::put), clears any existing limit
/// server-side (the gateway PUT is full-replace, so every field is always
/// sent).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrgConstraints {
    /// Hard cap (USD) on total monthly spend.
    pub cost_limit_monthly_usd: Option<f64>,
    /// Sliding window for the token rate limit, in seconds.
    pub token_window_seconds: Option<i64>,
    /// Max tokens allowed per `token_window_seconds`.
    pub max_tokens_per_window: Option<i64>,
    /// Max requests per minute per API key.
    pub max_requests_per_minute: Option<i64>,
}
