use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum QianjiError {
    #[error("Graph topology error: {0}")]
    TopologyError(String),

    #[error("Node execution failed: {0}")]
    ExecutionError(String),

    #[error("Strategic drift detected: {0}")]
    DriftError(String),

    #[error("Resource exhaustion: {0}")]
    CapacityError(String),
}
