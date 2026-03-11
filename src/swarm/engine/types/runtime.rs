use crate::error::QianjiError;
use crate::telemetry::PulseEmitter;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::SwarmAgentReport;

pub(in crate::swarm::engine) type WorkerJoinSet =
    tokio::task::JoinSet<Result<SwarmAgentReport, QianjiError>>;

#[derive(Debug, Clone)]
pub(in crate::swarm::engine) struct WorkerRuntimeConfig {
    pub(in crate::swarm::engine) session_id: String,
    pub(in crate::swarm::engine) redis_url: Option<String>,
    pub(in crate::swarm::engine) cluster_id: Option<String>,
    pub(in crate::swarm::engine) remote_enabled: bool,
    pub(in crate::swarm::engine) poll_interval_ms: u64,
    pub(in crate::swarm::engine) allow_local_affinity_proxy: bool,
    pub(in crate::swarm::engine) pulse_emitter: Option<Arc<dyn PulseEmitter>>,
}

pub(in crate::swarm::engine) fn generate_swarm_session_id() -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let random_suffix: u64 = rand::random();
    format!("swarm_{now_ms}_{random_suffix:x}")
}
