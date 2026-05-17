//! Typed handles for every Floopy-only gateway resource. Obtain them from
//! the [`Floopy`](crate::Floopy) client (e.g. `client.decisions()`).

mod constraints;
mod decisions;
mod evaluations;
mod experiments;
mod export;
mod feedback;
mod routing;
mod sessions;

pub use constraints::Constraints;
pub use decisions::Decisions;
pub use evaluations::Evaluations;
pub use experiments::Experiments;
pub use export::{DecisionExportStream, Export};
pub use feedback::Feedback;
pub use routing::Routing;
pub use sessions::Sessions;

use crate::error::{Error, Result};

/// Resources expect a body on every non-204 endpoint; treat an empty body
/// as a decode error.
pub(crate) fn require<T>(data: Option<T>) -> Result<T> {
    data.ok_or_else(|| Error::Decode("gateway returned an empty response body".to_owned()))
}
