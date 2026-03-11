use crate::telemetry::PulseEmitter;
use std::sync::Arc;

/// Swarm execution options.
#[derive(Debug, Clone)]
pub struct SwarmExecutionOptions {
    /// Shared session id for all workers. Auto-generated when not provided.
    pub session_id: Option<String>,
    /// Optional Valkey URL used for checkpoint/consensus synchronization.
    pub redis_url: Option<String>,
    /// Optional cluster id used by remote possession routing.
    pub cluster_id: Option<String>,
    /// Enables background remote-possession responder loop for each role worker.
    pub enable_remote_possession: bool,
    /// Poll interval for background remote-possession responder.
    pub possession_poll_interval_ms: u64,
    /// Allows manager/auditor workers to proxy missing roles when no global candidate exists.
    pub allow_local_affinity_proxy: bool,
    /// Optional pulse telemetry emitter used for non-blocking observability events.
    pub pulse_emitter: Option<Arc<dyn PulseEmitter>>,
}

impl Default for SwarmExecutionOptions {
    fn default() -> Self {
        Self {
            session_id: None,
            redis_url: None,
            cluster_id: None,
            enable_remote_possession: false,
            possession_poll_interval_ms: 500,
            allow_local_affinity_proxy: true,
            pulse_emitter: None,
        }
    }
}
