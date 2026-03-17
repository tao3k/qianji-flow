use super::services::SchedulerRuntimeServices;
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
        Self::with_runtime_services_config(
            engine,
            SchedulerAgentIdentity::from_env(),
            SchedulerRuntimeServices {
                consensus_manager,
                ..Default::default()
            },
        )
    }

    /// Creates a scheduler with optional consensus manager and explicit identity.
    #[must_use]
    pub fn with_consensus_manager_and_identity(
        engine: QianjiEngine,
        consensus_manager: Option<Arc<ConsensusManager>>,
        execution_identity: SchedulerAgentIdentity,
    ) -> Self {
        Self::with_runtime_services_config(
            engine,
            execution_identity,
            SchedulerRuntimeServices {
                consensus_manager,
                ..Default::default()
            },
        )
    }

    /// Creates a scheduler with explicit runtime services and inferred defaults.
    #[must_use]
    pub fn with_runtime_services(
        engine: QianjiEngine,
        consensus_manager: Option<Arc<ConsensusManager>>,
        remote_possession_bus: Option<Arc<RemotePossessionBus>>,
        cluster_id: Option<String>,
        execution_identity: SchedulerAgentIdentity,
    ) -> Self {
        Self::with_runtime_services_config(
            engine,
            execution_identity,
            SchedulerRuntimeServices {
                consensus_manager,
                remote_possession_bus,
                cluster_id,
                ..Default::default()
            },
        )
    }

    /// Creates a scheduler with the full runtime dependency bundle.
    #[must_use]
    pub fn with_runtime_services_config(
        engine: QianjiEngine,
        execution_identity: SchedulerAgentIdentity,
        services: SchedulerRuntimeServices,
    ) -> Self {
        let cluster_id = services
            .cluster_id
            .or_else(|| std::env::var("CLUSTER_ID").ok())
            .unwrap_or_else(|| "local_cluster".to_string());

        Self {
            engine: Arc::new(RwLock::new(engine)),
            max_total_steps: 1000,
            consensus_manager: services.consensus_manager,
            remote_possession_bus: services.remote_possession_bus,
            role_registry: services.role_registry,
            cluster_id,
            execution_identity,
            execution_policy: services.execution_policy,
            telemetry_emitter: services.telemetry_emitter,
        }
    }
}
