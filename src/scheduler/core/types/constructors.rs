use crate::consensus::ConsensusManager;
use crate::engine::QianjiEngine;
use crate::scheduler::identity::SchedulerAgentIdentity;
use crate::swarm::RemotePossessionBus;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

pub(super) const EXTERNAL_PROGRESS_WAIT_MS: u64 = 200;
pub(super) const EXTERNAL_PROGRESS_TIMEOUT_MS: u64 = 30_000;
pub(super) const REMOTE_POSSESSION_REQUEST_TTL_SECONDS: u64 = 120;
pub(super) const REMOTE_POSSESSION_MAX_WAIT_MS: u64 = 30_000;

pub(super) struct ConsensusCheckpointView<'a> {
    pub(super) session_id: Option<&'a str>,
    pub(super) redis_url: Option<&'a str>,
    pub(super) total_steps: u32,
    pub(super) active_branches: &'a HashSet<String>,
    pub(super) context: &'a serde_json::Value,
}

pub(super) enum ConsensusOutcome {
    Proceed(serde_json::Value),
    Suspend(serde_json::Value),
}

pub(super) enum RemoteDelegationOutcome {
    Noop,
    Progressed,
    Suspend(serde_json::Value),
}

/// Drives the parallel execution of the Qianji Box mechanisms.
pub struct QianjiScheduler {
    /// Thread-safe access to the underlying graph.
    pub(super) engine: Arc<RwLock<QianjiEngine>>,
    /// Maximum total execution steps to prevent runaway loops.
    pub(super) max_total_steps: u32,
    /// Optional manager for distributed consensus voting.
    pub(super) consensus_manager: Option<Arc<ConsensusManager>>,
    /// Optional remote possession transport for cross-cluster delegation.
    pub(super) remote_possession_bus: Option<Arc<RemotePossessionBus>>,
    /// Local cluster id used to avoid self-delegation loops.
    pub(super) cluster_id: String,
    /// Runtime execution identity used by role-aware scheduling.
    pub(super) execution_identity: SchedulerAgentIdentity,
}

impl QianjiScheduler {
    /// Creates a new scheduler for the given engine.
    #[must_use]
    pub fn new(engine: QianjiEngine) -> Self {
        Self::with_consensus_manager(engine, None)
    }

    /// Creates a new scheduler with optional distributed consensus manager.
    #[must_use]
    pub fn with_consensus_manager(
        engine: QianjiEngine,
        consensus_manager: Option<Arc<ConsensusManager>>,
    ) -> Self {
        Self::with_runtime_services(
            engine,
            consensus_manager,
            None,
            None,
            SchedulerAgentIdentity::from_env(),
        )
    }

    /// Creates a scheduler with optional distributed consensus manager and explicit
    /// execution identity for role-aware swarm routing.
    #[must_use]
    pub fn with_consensus_manager_and_identity(
        engine: QianjiEngine,
        consensus_manager: Option<Arc<ConsensusManager>>,
        execution_identity: SchedulerAgentIdentity,
    ) -> Self {
        Self::with_runtime_services(engine, consensus_manager, None, None, execution_identity)
    }

    /// Creates a scheduler with full runtime services, including optional cross-cluster
    /// possession bus used for remote role execution.
    #[must_use]
    pub fn with_runtime_services(
        engine: QianjiEngine,
        consensus_manager: Option<Arc<ConsensusManager>>,
        remote_possession_bus: Option<Arc<RemotePossessionBus>>,
        cluster_id: Option<String>,
        execution_identity: SchedulerAgentIdentity,
    ) -> Self {
        let cluster_id = cluster_id
            .or_else(|| std::env::var("CLUSTER_ID").ok())
            .unwrap_or_else(|| "local_cluster".to_string());
        Self {
            engine: Arc::new(RwLock::new(engine)),
            max_total_steps: 1000,
            consensus_manager,
            remote_possession_bus,
            cluster_id,
            execution_identity,
        }
    }
}
