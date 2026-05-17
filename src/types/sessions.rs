use async_openai::types::chat::ChatCompletionRequestMessage;
use serde::Deserialize;

/// Provenance for one reconstructed exchange.
#[derive(Debug, Clone, Deserialize)]
pub struct SessionTurn {
    /// The originating request id.
    pub request_id: String,
    /// RFC3339 timestamp of the originating request.
    pub created_at: String,
    /// Model used for the turn.
    pub model: String,
    /// Provider used for the turn.
    pub provider: String,
}

/// A conversation restored from Floopy's stored logs.
///
/// `messages` is chronological (oldest → newest) and is a drop-in for the
/// `messages` of a follow-up `chat().create(...)` call.
#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    /// The session id.
    pub session_id: String,
    /// Reconstructed messages, oldest first.
    pub messages: Vec<ChatCompletionRequestMessage>,
    /// Number of stored turns that contributed to `messages`.
    pub turn_count: i64,
    /// Stored turns that contributed to `messages`.
    pub turns: Vec<SessionTurn>,
}
