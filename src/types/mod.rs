//! Public, fully-typed data models for every Floopy-only resource. Wire
//! field names are snake_case and map 1:1 to Rust field names (no rename
//! attributes needed).

mod constraints;
mod decisions;
mod evaluations;
mod experiments;
mod export;
mod feedback;
mod routing;
mod sessions;

pub use constraints::OrgConstraints;
pub use decisions::{Decision, DecisionListPage, DecisionListParams};
pub use evaluations::{
    EvaluationCreateParams, EvaluationResultRow, EvaluationResultsPage, EvaluationRun,
};
pub use experiments::{
    Experiment, ExperimentCreateParams, ExperimentListPage, ExperimentListParams,
    ExperimentResults, VariantResults,
};
pub use export::{ExportDecisionsParams, ExportTrailer, ExportedDecisionRow};
pub use feedback::{FeedbackSubmitParams, FeedbackSubmitResponse};
pub use routing::{RoutingExplainParams, RoutingExplainResult};
pub use sessions::{Session, SessionTurn};
