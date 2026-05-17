use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures::stream::BoxStream;
use futures::{Stream, StreamExt};
use reqwest::Method;
use serde_json::Value;

use crate::constants::ENDPOINT_EXPORT_DECISIONS;
use crate::error::{Error, Result};
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::{ExportDecisionsParams, ExportTrailer, ExportedDecisionRow};

/// Streams the decision log.
pub struct Export {
    t: Arc<HttpTransport>,
}

fn is_trailer(value: &Value) -> bool {
    value.get("trailer").and_then(Value::as_bool) == Some(true)
}

fn parse_row(value: Value) -> Result<ExportedDecisionRow> {
    serde_json::from_value(value).map_err(|e| Error::Decode(e.to_string()))
}

impl Export {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Stream decision rows from the JSONL export. The terminal trailer
    /// record is skipped — use [`Export::decisions_with_trailer`] to
    /// capture it. The request is issued on first poll.
    pub fn decisions(
        &self,
        params: ExportDecisionsParams,
        req: Option<RequestOptions>,
    ) -> impl Stream<Item = Result<ExportedDecisionRow>> + Send + 'static {
        let t = self.t.clone();
        async_stream::try_stream! {
            let lines = t
                .stream_lines(
                    Method::GET,
                    ENDPOINT_EXPORT_DECISIONS,
                    &params.query(),
                    req.as_ref(),
                )
                .await?;
            futures::pin_mut!(lines);
            while let Some(line) = lines.next().await {
                let value = line?;
                if is_trailer(&value) {
                    continue;
                }
                yield parse_row(value)?;
            }
        }
    }

    /// Stream decision rows and capture the trailer. Drive the returned
    /// [`DecisionExportStream`] to completion, then call
    /// [`DecisionExportStream::trailer`] for the summary.
    pub fn decisions_with_trailer(
        &self,
        params: ExportDecisionsParams,
        req: Option<RequestOptions>,
    ) -> DecisionExportStream {
        let t = self.t.clone();
        let trailer: Arc<Mutex<Option<ExportTrailer>>> = Arc::new(Mutex::new(None));
        let sink = trailer.clone();
        let inner = async_stream::try_stream! {
            let lines = t
                .stream_lines(
                    Method::GET,
                    ENDPOINT_EXPORT_DECISIONS,
                    &params.query(),
                    req.as_ref(),
                )
                .await?;
            futures::pin_mut!(lines);
            while let Some(line) = lines.next().await {
                let value = line?;
                if is_trailer(&value) {
                    if let Ok(parsed) = serde_json::from_value::<ExportTrailer>(value) {
                        if let Ok(mut slot) = sink.lock() {
                            *slot = Some(parsed);
                        }
                    }
                    continue;
                }
                yield parse_row(value)?;
            }
        };
        DecisionExportStream {
            inner: inner.boxed(),
            trailer,
        }
    }
}

/// An iterable export that also captures the trailer record.
///
/// Implements [`Stream`] over [`ExportedDecisionRow`]. After the stream is
/// fully consumed, [`DecisionExportStream::trailer`] returns the summary
/// (truncation reason / totals), or `None` if the gateway sent no trailer.
pub struct DecisionExportStream {
    inner: BoxStream<'static, Result<ExportedDecisionRow>>,
    trailer: Arc<Mutex<Option<ExportTrailer>>>,
}

impl DecisionExportStream {
    /// The export trailer, or `None` if not yet seen / not sent.
    #[must_use]
    pub fn trailer(&self) -> Option<ExportTrailer> {
        self.trailer.lock().ok().and_then(|slot| slot.clone())
    }
}

impl Stream for DecisionExportStream {
    type Item = Result<ExportedDecisionRow>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}
