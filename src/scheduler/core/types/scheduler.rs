use crate::consensus::ConsensusManager;
use crate::engine::QianjiEngine;
use crate::scheduler::identity::SchedulerAgentIdentity;
use crate::scheduler::policy::{RoleAvailabilityRegistry, SchedulerExecutionPolicy};
use crate::swarm::RemotePossessionBus;
use crate::telemetry::PulseEmitter;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Drives the parallel execution of the Qianji Box mechanisms.
pub struct QianjiScheduler {
    /// Thread-safe access to the underlying graph.
    pub(in crate::scheduler::core) engine: Arc<RwLock<QianjiEngine>>,
    /// Maximum total execution steps to prevent runaway loops.
    pub(in crate::scheduler::core) max_total_steps: u32,
    /// Optional manager for distributed consensus voting.
    pub(in crate::scheduler::core) consensus_manager: Option<Arc<ConsensusManager>>,
    /// Optional remote possession transport for cross-cluster delegation.
    pub(in crate::scheduler::core) remote_possession_bus: Option<Arc<RemotePossessionBus>>,
    /// Optional global role availability registry used by affinity failover.
    pub(in crate::scheduler::core) role_registry: Option<Arc<dyn RoleAvailabilityRegistry>>,
    /// Local cluster id used to avoid self-delegation loops.
    pub(in crate::scheduler::core) cluster_id: String,
    /// Runtime execution identity used by role-aware scheduling.
    pub(in crate::scheduler::core) execution_identity: SchedulerAgentIdentity,
    /// Runtime execution policy for affinity and local delegation.
    pub(in crate::scheduler::core) execution_policy: SchedulerExecutionPolicy,
    /// Optional non-blocking telemetry emitter for swarm pulse events.
    pub(in crate::scheduler::core) telemetry_emitter: Option<Arc<dyn PulseEmitter>>,
}
