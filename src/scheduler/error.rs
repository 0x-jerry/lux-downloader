use crate::persistence::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("task not found")]
    NotFound,
    #[error("unsupported source")]
    UnsupportedSource,
    #[error("task is already running")]
    AlreadyRunning,
    #[error("invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
}
