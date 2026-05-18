//! Static configuration: defaults, Floopy wire headers, and API endpoints.
//!
//! Mirrors the constants in the Node/Python/Go SDKs so behaviour stays in
//! lockstep across languages.

use std::time::Duration;

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

/// Encode everything except the RFC3986 unreserved set (`A-Za-z0-9-._~`),
/// matching JS `encodeURIComponent`, Python `quote(safe="")`, and Go
/// `url.PathEscape` so a given id maps to the same path across all SDKs.
const PATH_SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

/// The public Floopy gateway base URL. Override with
/// [`crate::FloopyBuilder::base_url`] for self-hosted gateways.
pub const DEFAULT_BASE_URL: &str = "https://api.floopy.ai/v1";

/// Default per-request timeout when none is configured.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Default retry budget for transient failures.
pub const DEFAULT_MAX_RETRIES: u32 = 2;

pub(crate) const USER_AGENT_PREFIX: &str = "floopy-sdk";

// --- headers --------------------------------------------------------------

pub(crate) const HEADER_CACHE_ENABLED: &str = "Floopy-Cache-Enabled";
pub(crate) const HEADER_CACHE_BUCKET_MAX_SIZE: &str = "Floopy-Cache-Bucket-Max-Size";
pub(crate) const HEADER_PROMPT_ID: &str = "Floopy-Prompt-Id";
pub(crate) const HEADER_PROMPT_VERSION: &str = "Floopy-Prompt-Version";
pub(crate) const HEADER_LLM_SECURITY_ENABLED: &str = "floopy-llm-security-enabled";
pub(crate) const HEADER_FLOOPY_PROVIDER: &str = "floopy-provider";
pub(crate) const HEADER_CONFIRM: &str = "X-Floopy-Confirm";
pub(crate) const HEADER_REQUEST_ID: &str = "X-Request-Id";
pub(crate) const HEADER_FLOOPY_SDK: &str = "X-Floopy-SDK";

/// The `X-Floopy-Confirm` value the gateway requires on experiment
/// create/rollback (gateway control SEC-009). Injected automatically by the
/// experiments resource.
pub const CONFIRM_EXPERIMENTS: &str = "experiments";

// --- endpoints ------------------------------------------------------------

pub(crate) const ENDPOINT_FEEDBACK: &str = "/feedback";
pub(crate) const ENDPOINT_DECISIONS: &str = "/decisions";
pub(crate) const ENDPOINT_EXPERIMENTS: &str = "/experiments";
pub(crate) const ENDPOINT_CONSTRAINTS: &str = "/constraints";
pub(crate) const ENDPOINT_EXPORT_DECISIONS: &str = "/export/decisions";
pub(crate) const ENDPOINT_ROUTING_EXPLAIN: &str = "/routing/explain";
pub(crate) const ENDPOINT_EVALUATIONS: &str = "/evaluations";
pub(crate) const ENDPOINT_FILES: &str = "/files";
pub(crate) const ENDPOINT_BATCHES: &str = "/batches";

/// Percent-encode a single path segment (matches JS `encodeURIComponent` /
/// Python `urllib.parse.quote(safe="")`).
pub(crate) fn path_seg(value: &str) -> String {
    utf8_percent_encode(value, PATH_SEGMENT).to_string()
}

pub(crate) fn decision_by_id(id: &str) -> String {
    format!("{ENDPOINT_DECISIONS}/{}", path_seg(id))
}
pub(crate) fn session_by_id(id: &str) -> String {
    format!("/session/{}", path_seg(id))
}
pub(crate) fn experiment_results(id: &str) -> String {
    format!("{ENDPOINT_EXPERIMENTS}/{}/results", path_seg(id))
}
pub(crate) fn experiment_rollback(id: &str) -> String {
    format!("{ENDPOINT_EXPERIMENTS}/{}/rollback", path_seg(id))
}
pub(crate) fn evaluation_by_id(id: &str) -> String {
    format!("{ENDPOINT_EVALUATIONS}/{}", path_seg(id))
}
pub(crate) fn evaluation_results(id: &str) -> String {
    format!("{ENDPOINT_EVALUATIONS}/{}/results", path_seg(id))
}
pub(crate) fn evaluation_cancel(id: &str) -> String {
    format!("{ENDPOINT_EVALUATIONS}/{}/cancel", path_seg(id))
}
pub(crate) fn file_by_id(id: &str) -> String {
    format!("{ENDPOINT_FILES}/{}", path_seg(id))
}
pub(crate) fn file_content(id: &str) -> String {
    format!("{ENDPOINT_FILES}/{}/content", path_seg(id))
}
pub(crate) fn batch_by_id(id: &str) -> String {
    format!("{ENDPOINT_BATCHES}/{}", path_seg(id))
}
pub(crate) fn batch_cancel(id: &str) -> String {
    format!("{ENDPOINT_BATCHES}/{}/cancel", path_seg(id))
}
