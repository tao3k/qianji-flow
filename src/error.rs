use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum QianjiError {
    #[error("Graph topology error: {0}")]
    Topology(String),

    #[error("Node execution failed: {0}")]
    Execution(String),

    #[error("Strategic drift detected: {0}")]
    Drift(String),

    #[error("Resource exhaustion: {0}")]
    Capacity(String),

    #[error("Checkpoint persistence failed: {0}")]
    CheckpointError(String),

    #[error("Execution aborted: {0}")]
    Aborted(String),
}
