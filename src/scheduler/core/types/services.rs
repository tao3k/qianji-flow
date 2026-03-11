use crate::consensus::ConsensusManager;
use crate::scheduler::policy::{RoleAvailabilityRegistry, SchedulerExecutionPolicy};
use crate::swarm::RemotePossessionBus;
use crate::telemetry::PulseEmitter;
use std::sync::Arc;

/// Runtime dependency bundle for scheduler execution.
#[derive(Clone, Default)]
pub struct SchedulerRuntimeServices {
    /// Optional manager for distributed consensus voting.
    pub consensus_manager: Option<Arc<ConsensusManager>>,
    /// Optional remote possession transport for cross-cluster delegation.
    pub remote_possession_bus: Option<Arc<RemotePossessionBus>>,
    /// Optional global role availability registry used by affinity failover.
    pub role_registry: Option<Arc<dyn RoleAvailabilityRegistry>>,
    /// Optional local cluster id override.
    pub cluster_id: Option<String>,
    /// Execution policy for role affinity and local proxy behavior.
    pub execution_policy: SchedulerExecutionPolicy,
    /// Optional non-blocking telemetry emitter for swarm pulse events.
    pub telemetry_emitter: Option<Arc<dyn PulseEmitter>>,
}
